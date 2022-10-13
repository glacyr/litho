use lsp_types::request::InlayHintRefreshRequest;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod completion;
mod definition;
mod document;
mod hover;
mod inlay_hint;
mod printer;
mod report;
mod store;
mod util;
mod workspace;

use completion::CompletionProvider;
use definition::DefinitionProvider;
use document::Document;
use hover::HoverProvider;
use inlay_hint::InlayHintProvider;
use printer::Printer;
use store::Store;
use util::{index_to_position, line_col_to_offset, span_to_range};
use workspace::Workspace;

#[derive(Debug)]
struct Backend {
    client: Client,
    workspace: Mutex<Workspace>,
}

fn apply(mut source: String, change: TextDocumentContentChangeEvent) -> String {
    match change.range {
        Some(range) => {
            let start = line_col_to_offset(&source, range.start.line, range.start.character);
            let end = line_col_to_offset(&source, range.end.line, range.end.character);
            source.replace_range(start..end, &change.text);
        }
        None => {}
    }

    source
}

impl Backend {
    pub async fn check<'a>(&self, document: &Document) {
        self.client
            .publish_diagnostics(
                document.url().to_owned(),
                document.diagnostics().collect(),
                document.version(),
            )
            .await;

        let _ = self
            .client
            .send_request::<InlayHintRefreshRequest>(())
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        eprintln!("Root: {:#?}", params.root_uri);
        self.workspace.lock().await.populate_inflection();

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        eprintln!("Asking to watch files.");

        let _ = self
            .client
            .register_capability(vec![Registration {
                id: "watch-workspace".to_owned(),
                method: "workspace/didChangeWatchedFiles".to_owned(),
                register_options: Some(
                    serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![FileSystemWatcher {
                            kind: None,
                            glob_pattern: "**/*.graphql".to_owned(),
                        }],
                    })
                    .unwrap(),
                ),
            }])
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let url = params.text_document.uri;

        let mut workspace = self.workspace.lock().await;
        let document = workspace.populate_file_contents(
            url.clone(),
            Some(params.text_document.version),
            params.text_document.text.to_owned(),
        );

        self.check(document).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        eprintln!("Files: {:#?}", params);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let url = params.text_document.uri;

        let mut workspace = self.workspace.lock().await;
        let document =
            workspace.update_file_contents(url, Some(params.text_document.version), |source| {
                params
                    .content_changes
                    .into_iter()
                    .fold(source.to_owned(), |source, change| apply(source, change))
            });
        self.check(document).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let workspace = self.workspace.lock().await;
        let document = match workspace
            .store()
            .get(&params.text_document_position_params.text_document.uri)
        {
            Some(document) => document,
            None => return Ok(None),
        };

        Ok(HoverProvider::new(document, workspace.database())
            .hover(params.text_document_position_params.position))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let workspace = self.workspace.lock().await;
        let document = match workspace
            .store()
            .get(&params.text_document_position_params.text_document.uri)
        {
            Some(document) => document,
            None => return Ok(None),
        };

        Ok(DefinitionProvider::new(document, workspace.database())
            .goto_definition(params.text_document_position_params.position))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let workspace = self.workspace.lock().await;
        let document = match workspace
            .store()
            .get(&params.text_document_position.text_document.uri)
        {
            Some(document) => document,
            None => return Ok(None),
        };

        Ok(Some(
            CompletionProvider::new(document, workspace.database())
                .completion(params.text_document_position.position),
        ))
    }
}

impl Backend {
    pub async fn inlay_hint(&self, params: InlayHintParams) -> Result<Vec<InlayHint>> {
        let workspace = self.workspace.lock().await;
        let document = match workspace.store().get(&params.text_document.uri) {
            Some(document) => document,
            None => return Ok(vec![]),
        };

        Ok(InlayHintProvider::new(document, workspace.database())
            .inlay_hints()
            .collect())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        workspace: Mutex::new(Workspace::new()),
    })
    .custom_method("textDocument/inlayHint", Backend::inlay_hint)
    .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}

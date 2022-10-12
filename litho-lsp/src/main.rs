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

use completion::CompletionProvider;
use definition::DefinitionProvider;
use document::Document;
use hover::HoverProvider;
use inlay_hint::InlayHintProvider;
use printer::Printer;
use store::Store;
use util::{index_to_position, line_col_to_offset, span_to_range};
#[derive(Debug)]
struct Backend {
    client: Client,
    store: Mutex<Store>,
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
                Some(document.version()),
            )
            .await;

        let response = self
            .client
            .send_request::<InlayHintRefreshRequest>(())
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let url = params.text_document.uri;

        let mut store = self.store.lock().await;
        let document = store.insert(
            url.clone(),
            params.text_document.version,
            params.text_document.text.to_owned(),
        );

        self.check(document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let url = params.text_document.uri;

        let mut store = self.store.lock().await;
        let document = store.update(url, params.text_document.version, |source| {
            params
                .content_changes
                .into_iter()
                .fold(source.to_owned(), |source, change| apply(source, change))
        });
        self.check(document).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let store = self.store.lock().await;
        let document = match store.get(&params.text_document_position_params.text_document.uri) {
            Some(document) => document,
            None => return Ok(None),
        };

        Ok(HoverProvider::new(document).hover(params.text_document_position_params.position))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let store = self.store.lock().await;
        let document = match store.get(&params.text_document_position_params.text_document.uri) {
            Some(document) => document,
            None => return Ok(None),
        };

        Ok(DefinitionProvider::new(document)
            .goto_definition(params.text_document_position_params.position))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let store = self.store.lock().await;
        let document = match store.get(&params.text_document_position.text_document.uri) {
            Some(document) => document,
            None => return Ok(None),
        };

        Ok(Some(
            CompletionProvider::new(document).completion(params.text_document_position.position),
        ))
    }
}

impl Backend {
    pub async fn inlay_hint(&self, params: InlayHintParams) -> Result<Vec<InlayHint>> {
        let store = self.store.lock().await;
        let document = match store.get(&params.text_document.uri) {
            Some(document) => document,
            None => return Ok(vec![]),
        };

        Ok(InlayHintProvider::new(document).inlay_hints().collect())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        store: Mutex::new(Store::new()),
    })
    .custom_method("textDocument/inlayHint", Backend::inlay_hint)
    .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}

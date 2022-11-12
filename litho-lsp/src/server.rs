use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use super::{
    CompletionProvider, DefinitionProvider, FormattingProvider, HoverProvider, InlayHintProvider,
    ReferencesProvider, TextDocumentContentParams, Workspace,
};

pub struct Server {
    workspace: Arc<Mutex<Workspace>>,
}

impl Server {
    pub fn new(workspace: Workspace) -> Server {
        Server {
            workspace: Arc::new(Mutex::new(workspace)),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut workspace = self.workspace.lock().await;
            workspace.importer_register(&self.workspace).await;
        }

        self.workspace
            .lock()
            .await
            .mutate(|workspace| {
                workspace.populate_inflection();
                workspace.populate_scalars();

                if let Some(root_uri) = params.root_uri {
                    let _ = workspace.populate_root(root_uri);
                }
            })
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(
                        " \t".chars().into_iter().map(|c| c.to_string()).collect(),
                    ),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let _ = self
            .workspace
            .lock()
            .await
            .client()
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

        self.workspace
            .lock()
            .await
            .mutate(|workspace| {
                workspace.populate_file_contents(
                    url.clone(),
                    Some(params.text_document.version),
                    false,
                    params.text_document.text.to_owned(),
                )
            })
            .await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.workspace
            .lock()
            .await
            .mutate(|workspace| {
                for change in params.changes {
                    let _ = workspace.refresh_file(change.uri);
                }
            })
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let url = params.text_document.uri;

        let _ = self
            .workspace
            .lock()
            .await
            .mutate(|workspace| workspace.refresh_file(url));
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let url = params.text_document.uri;

        self.workspace
            .lock()
            .await
            .mutate(|workspace| {
                workspace.update_file_contents(url, Some(params.text_document.version), |source| {
                    params
                        .content_changes
                        .into_iter()
                        .fold(source.to_owned(), |source, change| {
                            Workspace::apply(source, change)
                        })
                })
            })
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document_position_params.text_document.uri) else {
            return Ok(None)
        };

        Ok(HoverProvider::new(document, workspace.database())
            .hover(params.text_document_position_params.position))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document_position_params.text_document.uri) else {
            return Ok(None)
        };

        Ok(DefinitionProvider::new(document, &workspace)
            .goto_definition(params.text_document_position_params.position))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document_position.text_document.uri) else {
            return Ok(None)
        };

        Ok(Some(
            CompletionProvider::new(document, &workspace)
                .completion(params.text_document_position.position),
        ))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document_position.text_document.uri) else {
            return Ok(None)
        };

        Ok(ReferencesProvider::new(document, &workspace)
            .references(params.text_document_position.position))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document.uri) else {
            return Ok(None)
        };

        Ok(Some(
            FormattingProvider::new(document, &workspace).formatting(),
        ))
    }
}

impl Server {
    pub async fn inlay_hint(&self, params: InlayHintParams) -> Result<Vec<InlayHint>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document.uri) else {
            return Ok(vec![])
        };

        Ok(InlayHintProvider::new(document, workspace.database())
            .inlay_hints()
            .collect())
    }

    pub async fn text_document_content(&self, params: TextDocumentContentParams) -> Result<String> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.url) else {
            return Err(Error::invalid_params("File doesn't exist."))
        };

        Ok(document.text().to_string())
    }
}

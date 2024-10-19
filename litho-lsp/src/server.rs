use std::sync::Arc;

use futures::lock::Mutex;
use lsp_types::*;

use super::{
    CompletionProvider, DefinitionProvider, FormattingProvider, HoverProvider, InlayHintProvider,
    ReferencesProvider, SourceRoot, TextDocumentContentParams, Workspace,
};

type Result<T> = std::result::Result<T, ()>;

pub struct Server<S> {
    source_root: S,
    workspace: Arc<Mutex<Workspace>>,
}

impl<S> Server<S>
where
    S: SourceRoot<Error = ()>,
{
    pub fn new(source_root: S, workspace: Arc<Mutex<Workspace>>) -> Server<S> {
        Server {
            source_root,
            workspace,
        }
    }

    pub fn populate_root(&self, workspace: &mut Workspace, url: Url) -> Result<()> {
        for url in self.source_root.walk(&url)? {
            let _ = self.populate_file(workspace, url);
        }

        Ok(())
    }

    pub fn populate_file(&self, workspace: &mut Workspace, url: Url) -> Result<()> {
        let text = self.source_root.read(&url)?;
        workspace.populate_file_contents(url, None, false, text);

        Ok(())
    }

    pub fn refresh_file(&self, workspace: &mut Workspace, url: Url) -> Result<()> {
        workspace.remove_file(&url);
        self.populate_file(workspace, url)
    }

    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut workspace = self.workspace.lock().await;
        workspace
            .mutate(|workspace| {
                workspace.populate_builtins();

                if let Some(root_uri) = params.root_uri {
                    let _ = self.populate_root(workspace, root_uri);
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

    pub async fn initialized(&self, _: InitializedParams) {}

    pub async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
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

    #[cfg(feature = "fs")]
    pub async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.workspace
            .lock()
            .await
            .mutate(|workspace| {
                for change in params.changes {
                    let _ = self.refresh_file(workspace, change.uri);
                }
            })
            .await;
    }

    #[cfg(feature = "fs")]
    pub async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let url = params.text_document.uri;

        let _ = self
            .workspace
            .lock()
            .await
            .mutate(|workspace| self.refresh_file(workspace, url));
    }

    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {
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

    pub async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let workspace = self.workspace.lock().await;
        let Some(document) =
            workspace.document(&params.text_document_position_params.text_document.uri)
        else {
            return Ok(None);
        };

        Ok(HoverProvider::new(document, workspace.database())
            .hover(params.text_document_position_params.position))
    }

    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let workspace = self.workspace.lock().await;
        let Some(document) =
            workspace.document(&params.text_document_position_params.text_document.uri)
        else {
            return Ok(None);
        };

        Ok(DefinitionProvider::new(document, &workspace)
            .goto_definition(params.text_document_position_params.position))
    }

    pub async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document_position.text_document.uri)
        else {
            return Ok(None);
        };

        Ok(Some(
            CompletionProvider::new(document, &workspace)
                .completion(params.text_document_position.position),
        ))
    }

    pub async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document_position.text_document.uri)
        else {
            return Ok(None);
        };

        Ok(ReferencesProvider::new(document, &workspace)
            .references(params.text_document_position.position))
    }

    pub async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document.uri) else {
            return Ok(None);
        };

        Ok(Some(
            FormattingProvider::new(document, &workspace).formatting(),
        ))
    }

    pub async fn inlay_hint(&self, params: InlayHintParams) -> Result<Vec<InlayHint>> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.text_document.uri) else {
            return Ok(vec![]);
        };

        Ok(InlayHintProvider::new(document, workspace.database())
            .inlay_hints()
            .collect())
    }

    pub async fn text_document_content(&self, params: TextDocumentContentParams) -> Result<String> {
        let workspace = self.workspace.lock().await;
        let Some(document) = workspace.document(&params.url) else {
            return Err(());
        };

        Ok(document.text().to_string())
    }
}

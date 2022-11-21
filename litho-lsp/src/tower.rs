use lsp_types::*;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::{Client, LanguageServer};

use super::sources::fs::FileSystem;
use super::{Server, TextDocumentContentParams, Workspace};

pub struct TowerServer {
    client: Client,
    server: Server<FileSystem, Mutex<Workspace>>,
}

impl TowerServer {
    pub fn new(client: Client, workspace: Workspace) -> TowerServer {
        let server = Server::new(FileSystem, Mutex::new(workspace));
        TowerServer { client, server }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TowerServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        eprintln!("Going to initialize.");

        self.server
            .initialize(params)
            .await
            .map_err(|_| Error::invalid_request())
    }

    async fn initialized(&self, params: InitializedParams) {
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

        eprintln!("initialized");

        self.server.initialized(params).await
    }

    async fn shutdown(&self) -> Result<()> {
        self.server
            .shutdown()
            .await
            .map_err(|_| Error::invalid_request())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.server.did_open(params).await
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.server.did_change_watched_files(params).await
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.server.did_close(params).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.server.did_change(params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.server
            .hover(params)
            .await
            .map_err(|_| Error::invalid_request())
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.server
            .goto_definition(params)
            .await
            .map_err(|_| Error::invalid_request())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.server
            .completion(params)
            .await
            .map_err(|_| Error::invalid_request())
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.server
            .references(params)
            .await
            .map_err(|_| Error::invalid_request())
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.server
            .formatting(params)
            .await
            .map_err(|_| Error::invalid_request())
    }
}

impl TowerServer {
    pub async fn inlay_hint(&self, params: InlayHintParams) -> Result<Vec<InlayHint>> {
        self.server
            .inlay_hint(params)
            .await
            .map_err(|_| Error::invalid_request())
    }

    pub async fn text_document_content(&self, params: TextDocumentContentParams) -> Result<String> {
        self.server
            .text_document_content(params)
            .await
            .map_err(|_| Error::invalid_request())
    }
}

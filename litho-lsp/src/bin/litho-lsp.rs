use std::sync::Arc;

use futures::channel::mpsc::channel;
use futures::future::join;
use futures::{FutureExt, StreamExt};
use litho_lsp::importer::{Coordinator, Importer};
use litho_lsp::Workspace;
use tower_lsp::LspService;

use tower::TowerServer;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (mut workers_sink, workers_stream) = channel(1024);

    let (service, socket) = LspService::build(move |client| {
        let (sender, receiver) = channel(1024);

        let workspace = Workspace::new(sender);

        let (importer, worker) = Importer::new(Arc::downgrade(&workspace));

        let server = TowerServer::new(client.clone(), workspace.clone());

        let coordinator = Coordinator::new(receiver, importer, move |url, diagnostics, version| {
            let client = client.clone();

            async move {
                client.publish_diagnostics(url, diagnostics, version).await;
            }
        });

        workers_sink
            .try_send(join(coordinator.work(), worker.work()))
            .unwrap();

        server
    })
    .custom_method("textDocument/inlayHint", TowerServer::inlay_hint)
    .custom_method("textDocument/content", TowerServer::text_document_content)
    .finish();

    join(
        tower_lsp::Server::new(stdin, stdout, socket).serve(service),
        workers_stream.for_each_concurrent(None, |future| future.map(|_| ())),
    )
    .await;
}

mod tower {
    use std::sync::Arc;

    use futures::lock::Mutex;
    use litho_lsp::sources::fs::FileSystem;
    use litho_lsp::{Server, TextDocumentContentParams, Workspace};
    use lsp_types::*;
    use tower_lsp::jsonrpc::{Error, Result};
    use tower_lsp::{Client, LanguageServer};

    pub struct TowerServer {
        client: Client,
        server: Server<FileSystem>,
    }

    impl TowerServer {
        pub fn new(client: Client, workspace: Arc<Mutex<Workspace>>) -> TowerServer {
            let server = Server::new(FileSystem, workspace);
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

        async fn formatting(
            &self,
            params: DocumentFormattingParams,
        ) -> Result<Option<Vec<TextEdit>>> {
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

        pub async fn text_document_content(
            &self,
            params: TextDocumentContentParams,
        ) -> Result<String> {
            self.server
                .text_document_content(params)
                .await
                .map_err(|_| Error::invalid_request())
        }
    }
}

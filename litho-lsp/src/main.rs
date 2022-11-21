use futures::future::join;
use tower_lsp::LspService;

mod completion;
mod definition;
mod diagnostic;
mod document;
mod formatting;
mod hover;
pub mod importer;
mod inlay_hint;
mod printer;
mod references;
mod server;
pub mod sources;
mod store;
mod text_document_content;
mod tower;
mod workspace;

use completion::CompletionProvider;
use definition::DefinitionProvider;
use document::Document;
use formatting::FormattingProvider;
use hover::HoverProvider;
pub use importer::{Importer, ImporterCallback};
use inlay_hint::InlayHintProvider;
use printer::Printer;
use references::ReferencesProvider;
use server::Server;
pub use sources::SourceRoot;
use store::Store;
use text_document_content::TextDocumentContentParams;
use tower::TowerServer;
use workspace::Workspace;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (mut pool, pool_worker) = importer::threaded::ImporterPool::new();

    let (service, socket) = LspService::build(move |client| {
        TowerServer::new(
            client.clone(),
            Workspace::new(
                Box::new(move |url, diagnostics, version| {
                    let client = client.clone();

                    Box::pin(async move {
                        let _ = client.publish_diagnostics(url, diagnostics, version).await;
                    })
                }),
                pool.importer(),
            ),
        )
    })
    .custom_method("textDocument/inlayHint", TowerServer::inlay_hint)
    .custom_method("textDocument/content", TowerServer::text_document_content)
    .finish();

    join(
        tower_lsp::Server::new(stdin, stdout, socket).serve(service),
        pool_worker.work(),
    )
    .await;
}

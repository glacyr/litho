use futures::future::join;
use tower_lsp::LspService;

mod completion;
mod definition;
mod diagnostic;
mod document;
mod formatting;
mod hover;
mod importer;
mod inlay_hint;
mod paths;
mod printer;
mod references;
mod server;
mod store;
mod text_document_content;
mod workspace;

use completion::CompletionProvider;
use definition::DefinitionProvider;
use document::Document;
use formatting::FormattingProvider;
use hover::HoverProvider;
use importer::{ImportQueue, Importer};
use inlay_hint::InlayHintProvider;
use paths::url_from_path;
use printer::Printer;
use references::ReferencesProvider;
use server::Server;
use store::Store;
use text_document_content::TextDocumentContentParams;
use workspace::Workspace;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (mut queue, queue_worker) = ImportQueue::new();

    let (service, socket) =
        LspService::build(move |client| Server::new(Workspace::new(client, queue.importer())))
            .custom_method("textDocument/inlayHint", Server::inlay_hint)
            .custom_method("textDocument/content", Server::text_document_content)
            .finish();

    join(
        tower_lsp::Server::new(stdin, stdout, socket).serve(service),
        queue_worker,
    )
    .await;
}

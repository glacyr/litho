use tower_lsp::LspService;

mod completion;
mod definition;
mod diagnostic;
mod document;
mod hover;
mod inlay_hint;
mod paths;
mod printer;
mod references;
mod server;
mod store;
mod workspace;

use completion::CompletionProvider;
use definition::DefinitionProvider;
use document::Document;
use hover::HoverProvider;
use inlay_hint::InlayHintProvider;
use paths::url_from_path;
use printer::Printer;
use references::ReferencesProvider;
use server::Server;
use store::Store;
use workspace::Workspace;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Server::new(Workspace::new(client)))
        .custom_method("textDocument/inlayHint", Server::inlay_hint)
        .finish();
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}

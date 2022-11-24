use futures::channel::mpsc::{channel, Receiver};
use futures::StreamExt;
use litho_lsp::{Server, Workspace, WorkspaceUpdate};
use lsp_types::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
pub enum Request {
    Initialize(InitializeParams),
    DidOpen(DidOpenTextDocumentParams),
    DidChange(DidChangeTextDocumentParams),
    Completion(CompletionParams),
}

#[derive(Serialize)]
pub enum Response {
    Initialize(InitializeResult),
    DidOpen(()),
    DidChange(()),
    Completion(Option<CompletionResponse>),
}

#[derive(Serialize)]
pub struct DiagnosticsUpdate {
    url: Url,
    diagnostics: Vec<Diagnostic>,
    version: Option<i32>,
}

#[wasm_bindgen]
extern "C" {
    pub fn update_diagnostics(update: &str);
}

pub struct WasmServer {
    server: Server<()>,
    receiver: Receiver<WorkspaceUpdate>,
}

#[wasm_bindgen]
pub fn create_server() -> *mut WasmServer {
    let (sender, receiver) = channel(1024);

    let workspace = Workspace::new(sender);
    let server = Server::new((), workspace);

    Box::into_raw(Box::new(WasmServer { server, receiver }))
}

#[wasm_bindgen]
pub async unsafe fn server_work(server: *mut WasmServer) {
    let server = server.as_mut().unwrap();

    while let Some(update) = server.receiver.next().await {
        match update {
            WorkspaceUpdate::Diagnostics {
                url,
                diagnostics,
                version,
            } => {
                let update = DiagnosticsUpdate {
                    url,
                    diagnostics,
                    version,
                };
                update_diagnostics(&serde_json::to_string(&update).unwrap());
            }
            WorkspaceUpdate::Imports { .. } => {}
        }
    }
}

#[wasm_bindgen]
pub async unsafe fn server_message(server: *mut WasmServer, request: &str) -> String {
    let server = &mut server.as_mut().unwrap().server;

    let request: Request = serde_json::from_str(request).unwrap();
    let response = match request {
        Request::Initialize(params) => {
            let result = server.initialize(params).await.unwrap();
            Response::Initialize(result)
        }
        Request::DidOpen(params) => {
            server.did_open(params).await;
            Response::DidOpen(())
        }
        Request::DidChange(params) => {
            server.did_change(params).await;
            Response::DidChange(())
        }
        Request::Completion(params) => {
            let response = server.completion(params).await.unwrap();
            Response::Completion(response)
        }
    };

    serde_json::to_string(&response).unwrap()
}

#[wasm_bindgen]
pub fn destroy_server(server: *mut WasmServer) {
    unsafe { Box::from_raw(server) };
}

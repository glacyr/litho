use futures::lock::Mutex;
use litho_lsp::{Server, Workspace};
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

#[wasm_bindgen]
pub fn create_server() -> *mut Server<()> {
    let workspace = Workspace::new(
        Box::new(|url, diagnostics, version| {
            Box::pin(async move {
                let update = DiagnosticsUpdate {
                    url,
                    diagnostics,
                    version,
                };
                update_diagnostics(&serde_json::to_string(&update).unwrap());
            })
        }),
        (),
    );
    let server = Server::new((), Mutex::new(workspace));

    Box::into_raw(Box::new(server))
}

#[wasm_bindgen]
pub async unsafe fn server_message(server: *mut Server<()>, request: &str) -> String {
    let request: Request = serde_json::from_str(request).unwrap();
    let response = match request {
        Request::Initialize(params) => {
            let result = server.as_mut().unwrap().initialize(params).await.unwrap();
            Response::Initialize(result)
        }
        Request::DidOpen(params) => {
            server.as_mut().unwrap().did_open(params).await;
            Response::DidOpen(())
        }
        Request::DidChange(params) => {
            server.as_mut().unwrap().did_change(params).await;
            Response::DidChange(())
        }
        Request::Completion(params) => {
            let response = server.as_mut().unwrap().completion(params).await.unwrap();
            Response::Completion(response)
        }
    };

    serde_json::to_string(&response).unwrap()
}

#[wasm_bindgen]
pub fn destroy_server(server: *mut Server<()>) {
    unsafe { Box::from_raw(server) };
}

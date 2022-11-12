use lsp_types::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TextDocumentContentParams {
    pub url: Url,
}

use std::sync::Arc;

use litho_language::lex::SourceId;
use litho_language::Document as Ast;
use lsp_types::Url;
use smol_str::SmolStr;

#[derive(Debug)]
pub struct Document {
    source_id: SourceId,
    url: Url,
    version: Option<i32>,
    internal: bool,
    text: SmolStr,
    pub(crate) ast: Option<Arc<Ast<SmolStr>>>,
}

impl Document {
    pub fn new(
        source_id: SourceId,
        url: Url,
        version: Option<i32>,
        internal: bool,
        text: &str,
    ) -> Document {
        Document {
            source_id,
            url,
            version,
            internal,
            text: text.into(),
            ast: None,
        }
    }

    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn version(&self) -> Option<i32> {
        self.version
    }

    pub fn is_internal(&self) -> bool {
        self.internal
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn ast(&self) -> &Ast<SmolStr> {
        self.ast.as_ref().unwrap()
    }
}

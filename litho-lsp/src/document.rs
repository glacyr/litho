use litho_language::chk::collect_errors;
use litho_language::lex::{SourceId, Span};
use litho_language::{Document as Ast, Parse};
use smol_str::SmolStr;
use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::diagnostic::serialize_diagnostic;

use super::Workspace;

#[derive(Debug)]
pub struct Document {
    url: Url,
    version: Option<i32>,
    internal: bool,
    text: SmolStr,
    diagnostics: Vec<litho_diagnostics::Diagnostic<Span>>,
    ast: Ast<SmolStr>,
}

impl Document {
    pub fn new(
        source_id: SourceId,
        url: Url,
        version: Option<i32>,
        internal: bool,
        text: &str,
    ) -> Document {
        let result = Ast::parse_from_str(source_id, text).unwrap_or_default();

        Document {
            url,
            version,
            internal,
            text: text.into(),
            diagnostics: collect_errors(&result),
            ast: result.0,
        }
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
        &self.ast
    }

    pub fn diagnostics<'a>(
        &'a self,
        workspace: &'a Workspace,
    ) -> impl Iterator<Item = Diagnostic> + 'a {
        self.diagnostics
            .iter()
            .cloned()
            .chain(litho_validation::check(self.ast(), workspace.database()).into_iter())
            .map(|diagnostic| serialize_diagnostic(diagnostic, workspace))
    }
}

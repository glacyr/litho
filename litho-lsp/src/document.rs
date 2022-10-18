use litho_language::chk::{collect_errors, IntoReport};
use litho_language::lex::SourceId;
use litho_language::{Document as Ast, Parse};
use smol_str::SmolStr;
use tower_lsp::lsp_types::{Diagnostic, Url};

use super::{ReportBuilder, Workspace};

#[derive(Debug)]
pub struct Document {
    url: Url,
    version: Option<i32>,
    internal: bool,
    text: SmolStr,
    reports: Vec<ReportBuilder>,
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
            reports: collect_errors(&result)
                .into_iter()
                .map(IntoReport::into_report::<ReportBuilder>)
                .collect(),
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
        self.reports
            .iter()
            .cloned()
            .chain(
                litho_validation::check(self.ast(), workspace.database())
                    .into_iter()
                    .map(IntoReport::into_report::<ReportBuilder>),
            )
            .map(|report| report.into_diagnostic(workspace))
    }
}

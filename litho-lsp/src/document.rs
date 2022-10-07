use litho_language::chk::{collect_errors, IntoReport};
use litho_language::{Document as Ast, Parse};
use smol_str::SmolStr;
use tower_lsp::lsp_types::{Diagnostic, Url};

use super::report::ReportBuilder;

#[derive(Debug)]
pub struct Document {
    url: Url,
    version: i32,
    text: SmolStr,
    reports: Vec<ReportBuilder>,
    ast: Ast<SmolStr>,
}

impl Document {
    pub fn new(url: Url, version: i32, text: &str) -> Document {
        let result = Ast::parse_from_str(0, text).unwrap_or_default();

        Document {
            url,
            version,
            text: text.into(),
            reports: collect_errors(&result)
                .into_iter()
                .map(|error| error.into_report::<ReportBuilder>())
                .collect(),
            ast: result.0,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn ast(&self) -> &Ast<SmolStr> {
        &self.ast
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = Diagnostic> + '_ {
        self.reports
            .iter()
            .cloned()
            .map(|report| report.into_diagnostic(self.url.clone(), self.text.as_ref()))
    }
}

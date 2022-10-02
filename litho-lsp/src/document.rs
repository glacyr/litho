use litho_language::chk::{Errors, IntoReport};
use litho_language::{Document as Ast, Parse};
use tower_lsp::lsp_types::{Diagnostic, Url};

use super::report::ReportBuilder;

#[derive(Debug)]
pub struct Document<'a> {
    url: Url,
    version: i32,
    text: &'a str,
    reports: Vec<ReportBuilder>,
    ast: Ast<'a>,
}

impl<'a> Document<'a> {
    pub fn new(url: Url, version: i32, text: &'a str) -> Document<'a> {
        let result = Ast::parse_from_str(0, text).unwrap_or_default();

        Document {
            url,
            version,
            text,
            reports: result
                .errors()
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

    pub fn ast(&self) -> &Ast<'a> {
        &self.ast
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = Diagnostic> + '_ {
        self.reports
            .iter()
            .cloned()
            .map(|report| report.into_diagnostic(self.url.clone(), self.text))
    }
}

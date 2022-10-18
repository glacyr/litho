use litho_language::lex::Span;
use tower_lsp::lsp_types::*;

use super::Workspace;

#[derive(Clone, Debug)]
pub struct LabelBuilder {
    span: Span,
    message: Option<String>,
}

impl litho_language::chk::LabelBuilder for LabelBuilder {
    type Label = Self;

    fn new(span: Span) -> Self {
        LabelBuilder {
            span,
            message: None,
        }
    }

    fn with_message<M>(mut self, msg: M) -> Self
    where
        M: ToString,
    {
        self.message = Some(msg.to_string());
        self
    }

    fn with_color(self, color: litho_language::ariadne::Color) -> Self {
        self
    }

    fn finish(self) -> Self::Label {
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReportBuilder {
    span: Span,
    code: Option<String>,
    message: Option<String>,
    labels: Vec<LabelBuilder>,
}

impl ReportBuilder {
    pub fn into_diagnostic(self, workspace: &Workspace) -> Diagnostic {
        Diagnostic {
            source: Some("litho".to_owned()),
            code: self.code.map(NumberOrString::String),
            message: self.message.unwrap_or_default(),
            range: workspace.span_to_range(self.span).unwrap_or_default(),
            related_information: Some(
                self.labels
                    .into_iter()
                    .flat_map(|label| {
                        workspace.span_to_location(label.span).map(|location| {
                            DiagnosticRelatedInformation {
                                location,
                                message: label.message.unwrap_or_default(),
                            }
                        })
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }
}

impl litho_language::chk::ReportBuilder for ReportBuilder {
    type Report = Self;
    type LabelBuilder = LabelBuilder;

    fn new(kind: litho_language::ariadne::ReportKind, span: Span) -> Self {
        ReportBuilder {
            span,
            ..Default::default()
        }
    }

    fn with_code<C>(mut self, code: C) -> Self
    where
        C: std::fmt::Display,
    {
        self.code = Some(format!("{}", code));
        self
    }

    fn with_message<M>(mut self, msg: M) -> Self
    where
        M: ToString,
    {
        self.message = Some(msg.to_string());
        self
    }

    fn with_help<N>(self, note: N) -> Self
    where
        N: ToString,
    {
        self
    }

    fn with_label(mut self, label: Self::LabelBuilder) -> Self {
        self.labels.push(label);
        self
    }

    fn finish(self) -> Self::Report {
        self
    }
}

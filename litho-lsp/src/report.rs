use line_col::LineColLookup;
use litho_language::lex::Span;
use tower_lsp::lsp_types::*;

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
    pub fn into_diagnostic(self, url: Url, source: &str) -> Diagnostic {
        let index = LineColLookup::new(source);
        let start = index.get(self.span.start);
        let end = index.get(self.span.end);

        Diagnostic {
            source: Some("litho".to_owned()),
            code: self.code.map(NumberOrString::String),
            message: self.message.unwrap_or_default(),
            range: Range::new(
                Position::new(start.0 as u32 - 1, start.1 as u32 - 1),
                Position::new(end.0 as u32 - 1, end.1 as u32 - 1),
            ),
            related_information: Some(
                self.labels
                    .into_iter()
                    .map(|label| {
                        let start = index.get(label.span.start);
                        let end = index.get(label.span.end);

                        DiagnosticRelatedInformation {
                            location: Location {
                                uri: url.clone(),
                                range: Range::new(
                                    Position::new(start.0 as u32 - 1, start.1 as u32 - 1),
                                    Position::new(end.0 as u32 - 1, end.1 as u32 - 1),
                                ),
                            },
                            message: label.message.unwrap_or_default(),
                        }
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

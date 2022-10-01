use std::collections::HashMap;
use std::sync::Mutex;

use line_col::LineColLookup;
use litho_language::chk::{Errors, IntoReport};
use litho_language::lex::Span;
use litho_language::{Document, Parse};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Mutex<HashMap<String, String>>,
}

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

#[derive(Default)]
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

fn line_col_to_offset(source: &str, line: u32, col: u32) -> usize {
    let line_offset = source
        .split_inclusive("\n")
        .take(line as usize)
        .fold(0, |sum, line| sum + line.len());
    line_offset + col as usize
}

fn apply(mut source: String, change: TextDocumentContentChangeEvent) -> String {
    match change.range {
        Some(range) => {
            let start = line_col_to_offset(&source, range.start.line, range.start.character);
            let end = line_col_to_offset(&source, range.end.line, range.end.character);
            source.replace_range(start..end, &change.text);
        }
        None => {}
    }

    source
}

impl Backend {
    pub async fn check(&self, url: Url, source: &str, version: i32) {
        let result = Document::parse_from_str(0, source).unwrap();
        let errors = result.errors();

        self.client
            .publish_diagnostics(
                url.clone(),
                errors
                    .into_iter()
                    .map(|error| {
                        error
                            .into_report::<ReportBuilder>()
                            .into_diagnostic(url.clone(), source)
                    })
                    .collect(),
                Some(version),
            )
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let url = params.text_document.uri;

        let source = {
            let mut documents = self.documents.lock().unwrap();
            documents.insert(url.to_string(), params.text_document.text.to_owned());
            params.text_document.text
        };

        self.check(url, &source, params.text_document.version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let url = params.text_document.uri;

        let source = {
            let mut documents = self.documents.lock().unwrap();
            let source = documents.entry(url.to_string()).or_default();
            let result = params
                .content_changes
                .into_iter()
                .fold(source.to_owned(), |source, change| apply(source, change));

            let _ = std::mem::replace(source, result.to_owned());
            result
        };

        self.check(url, &source, params.text_document.version).await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Mutex::new(HashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

use ariadne::{ColorGenerator, ReportKind};

use crate::ast::{Missing, MissingToken};
use crate::lex::Token;

use super::diagnostics::{LabelBuilder, ReportBuilder};
use super::IntoReport;

#[derive(Debug)]
pub enum Error<'ast, 'a> {
    UnrecognizedTokens { tokens: Vec<Token<'a>> },
    Recoverable(&'ast MissingToken),
}

impl<'ast, 'a> IntoReport for Error<'ast, 'a> {
    fn into_report<R>(self) -> R::Report
    where
        R: ReportBuilder,
    {
        let mut colors = ColorGenerator::new();

        match self {
            Error::UnrecognizedTokens { tokens } => {
                let mut span = tokens[0].span();
                span.end = tokens.last().unwrap().span().end;

                R::new(ReportKind::Error, span.source_id, span.start)
                    .with_code("E0001")
                    .with_message("Encountered unrecognized tokens.")
                    .with_label(
                        R::LabelBuilder::new(span)
                            .with_message("Couldn't parse these tokens.")
                            .with_color(colors.next()),
                    )
                    .finish()
            }
            Error::Recoverable(MissingToken { span, missing }) => match missing {
                Missing::Unknown => R::new(ReportKind::Error, span.source_id, span.start)
                    .with_code("E0002")
                    .with_message("Expected one or more tokens here.")
                    .with_label(
                        R::LabelBuilder::new(*span)
                            .with_message("Expected one or more tokens here.")
                            .with_color(colors.next()),
                    )
                    .finish(),
                Missing::Simple(title, label) => {
                    R::new(ReportKind::Error, span.source_id, span.start)
                        .with_code("E0003")
                        .with_message(title)
                        .with_label(
                            R::LabelBuilder::new(*span)
                                .with_message(label)
                                .with_color(colors.next()),
                        )
                        .finish()
                }
                Missing::Delimiter(title, start_label, start_span, label) => {
                    R::new(ReportKind::Error, span.source_id, span.start)
                        .with_code("E0004")
                        .with_message(title)
                        .with_label(
                            R::LabelBuilder::new(*start_span)
                                .with_message(start_label)
                                .with_color(colors.next()),
                        )
                        .with_label(
                            R::LabelBuilder::new(*span)
                                .with_message(label)
                                .with_color(colors.next()),
                        )
                        .finish()
                }
            },
        }
    }
}

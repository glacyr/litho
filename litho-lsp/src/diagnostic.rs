use litho_language::lex::Span;
use lsp_types::*;

use super::Workspace;

pub fn serialize_diagnostic(
    diagnostic: &litho_diagnostics::Diagnostic<Span>,
    workspace: &Workspace,
) -> Diagnostic {
    Diagnostic {
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("litho".to_owned()),
        code: Some(NumberOrString::String(diagnostic.code().to_owned())),
        message: diagnostic.message().to_owned(),
        range: workspace
            .span_to_range(diagnostic.span())
            .unwrap_or_default(),
        related_information: Some(
            diagnostic
                .labels()
                .into_iter()
                .flat_map(|(span, message)| {
                    workspace
                        .span_to_location(span)
                        .map(|location| DiagnosticRelatedInformation {
                            location,
                            message: message.to_owned(),
                        })
                })
                .collect(),
        ),
        ..Default::default()
    }
}

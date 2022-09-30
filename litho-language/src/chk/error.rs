use ariadne::{ColorGenerator, Label, Report, ReportKind};

use crate::ast::*;
use crate::lex::{Span, Token};

use super::diagnostics::{LabelBuilder, ReportBuilder};
use super::IntoReport;

#[derive(Debug)]
pub enum Error<'ast, 'a> {
    UnrecognizedTokens { tokens: &'ast Vec<Token<'a>> },
    // UnclosedVariableDefinitions {
    //     operation_definition: &'ast OperationDefinition<'a>,
    //     variable_definitions: &'ast VariableDefinitions<'a>,
    // },
    // VariableMissingDollarSign {
    //     variable: &'ast Variable<'a>,
    // },
    // UnclosedArguments {
    //     arguments: &'ast Arguments<'a>,
    // },
    // UnclosedSelectionSet {
    //     selection_set: &'ast SelectionSet<'a>,
    // },
    // UnclosedListValue {
    //     list_value: &'ast ListValue<'a>,
    // },
    // UnclosedObjectValue {
    //     object_value: &'ast ObjectValue<'a>,
    // },
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
            } //     Error::UnclosedVariableDefinitions {
              //         operation_definition,
              //         variable_definitions,
              //     } => {
              //         let span = variable_definitions.parens.0.span();

              //         R::new(ReportKind::Error, span.source_id, span.start)
              //             .with_code("E0002")
              //             .with_message("Variable definitions are missing closing parenthesis.")
              //             .with_label(
              //                 R::LabelBuilder::new(span)
              //                     .with_message("This `(` here ...")
              //                     .with_color(colors.next()),
              //             )
              //             .with_label(
              //                 R::LabelBuilder::new(variable_definitions.span().collapse_to_end())
              //                     .with_message("... should have a corresponding `)` here.")
              //                     .with_color(colors.next()),
              //             )
              //             .finish()
              //     }
              //     Error::VariableMissingDollarSign { variable } => {
              //         let span = variable.name.span();

              //         R::new(ReportKind::Error, span.source_id, span.start)
              //             .with_code("E0003")
              //             .with_message("Variable should be prefixed by `$`.")
              //             .with_label(
              //                 R::LabelBuilder::new(span)
              //                     .with_message(
              //                         "Variable defined here should be prefixed by `$` but isn't.",
              //                     )
              //                     .with_color(colors.next()),
              //             )
              //             .with_help(format!(
              //                 "Replace `{0}` with `${0}`.",
              //                 variable.name.as_ref(),
              //             ))
              //             .finish()
              //     }
              //     Error::UnclosedArguments { arguments } => {
              //         let span = arguments.parens.0.span();

              //         R::new(ReportKind::Error, span.source_id, span.start)
              //             .with_code("E0004")
              //             .with_message("Arguments are missing closing parenthesis.")
              //             .with_label(
              //                 R::LabelBuilder::new(span)
              //                     .with_message("This `(` here ...")
              //                     .with_color(colors.next()),
              //             )
              //             .with_label(
              //                 R::LabelBuilder::new(arguments.span().collapse_to_end())
              //                     .with_message("... should have a corresponding `)` here.")
              //                     .with_color(colors.next()),
              //             )
              //             .finish()
              //     }
              //     Error::UnclosedSelectionSet { selection_set } => {
              //         let span = selection_set.braces.0.span();

              //         R::new(ReportKind::Error, span.source_id, span.start)
              //             .with_code("E0005")
              //             .with_message("Selection set is missing closing brace.")
              //             .with_label(
              //                 R::LabelBuilder::new(span)
              //                     .with_message("This `{` here ...")
              //                     .with_color(colors.next()),
              //             )
              //             .with_label(
              //                 R::LabelBuilder::new(selection_set.span().collapse_to_end())
              //                     .with_message("... should have a corresponding `}` here.")
              //                     .with_color(colors.next()),
              //             )
              //             .finish()
              //     }
              //     Error::UnclosedListValue { list_value } => {
              //         let span = list_value.brackets.0.span();

              //         R::new(ReportKind::Error, span.source_id, span.start)
              //             .with_code("E0006")
              //             .with_message("List value is missing closing bracket.")
              //             .with_label(
              //                 R::LabelBuilder::new(span)
              //                     .with_message("This `[` here ...")
              //                     .with_color(colors.next()),
              //             )
              //             .with_label(
              //                 R::LabelBuilder::new(list_value.span().collapse_to_end())
              //                     .with_message("... should have a corresponding `]` here.")
              //                     .with_color(colors.next()),
              //             )
              //             .finish()
              //     }
              //     Error::UnclosedObjectValue { object_value } => {
              //         let span = object_value.braces.0.span();

              //         R::new(ReportKind::Error, span.source_id, span.start)
              //             .with_code("E0007")
              //             .with_message("Object value is missing closing brace.")
              //             .with_label(
              //                 R::LabelBuilder::new(span)
              //                     .with_message("This `{` here ...")
              //                     .with_color(colors.next()),
              //             )
              //             .with_label(
              //                 R::LabelBuilder::new(object_value.span().collapse_to_end())
              //                     .with_message("... should have a corresponding `}` here.")
              //                     .with_color(colors.next()),
              //             )
              //             .finish()
              //     }
        }
    }
}

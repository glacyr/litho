mod diagnostics;
mod error;

pub use diagnostics::{IntoReport, LabelBuilder, ReportBuilder};
pub use error::Error;

use crate::ast::{Node, Visit};

pub trait Errors<'a> {
    fn errors<'ast>(&'ast self) -> Vec<Error<'ast, 'a>>;
}

impl<'a, T> Errors<'a> for T
where
    T: Node<'a>,
{
    fn errors<'ast>(&'ast self) -> Vec<Error<'ast, 'a>> {
        let mut errors = vec![];
        self.traverse(&CollectErrors, &mut errors);
        errors
    }
}

pub struct CollectErrors;

impl<'ast, 'a> Visit<'ast, 'a> for CollectErrors
where
    'a: 'ast,
{
    type Accumulator = Vec<Error<'ast, 'a>>;

    fn visit_recoverable<T>(
        &self,
        node: &'ast crate::Recoverable<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node {
            Err(tokens) if !tokens.is_empty() => {
                accumulator.push(Error::UnrecognizedTokens { tokens })
            }
            Ok(_) | Err(_) => {}
        }
    }

    fn visit_operation_definition(
        &self,
        node: &'ast crate::OperationDefinition<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.variable_definitions.as_ref() {
            Some(defs) if defs.parens.1.is_err() => {
                accumulator.push(Error::UnclosedVariableDefinitions {
                    operation_definition: node,
                    variable_definitions: defs,
                })
            }
            _ => {}
        }
    }

    fn visit_variable(&self, node: &'ast crate::Variable<'a>, accumulator: &mut Self::Accumulator) {
        if node.dollar.is_err() {
            accumulator.push(Error::VariableMissingDollarSign { variable: node })
        }
    }

    fn visit_arguments(
        &self,
        node: &'ast crate::Arguments<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.parens.1.is_err() {
            accumulator.push(Error::UnclosedArguments { arguments: node })
        }
    }

    fn visit_selection_set(
        &self,
        selection_set: &'ast crate::SelectionSet<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if selection_set.braces.1.is_err() {
            accumulator.push(Error::UnclosedSelectionSet { selection_set })
        }
    }

    fn visit_list_value(
        &self,
        list_value: &'ast crate::ListValue<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if list_value.brackets.1.is_err() {
            accumulator.push(Error::UnclosedListValue { list_value })
        }
    }

    fn visit_object_value(
        &self,
        object_value: &'ast crate::ObjectValue<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if object_value.braces.1.is_err() {
            accumulator.push(Error::UnclosedObjectValue { object_value })
        }
    }
}

use std::collections::HashSet;
use std::hash::Hash;
use std::iter::once;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariablesAreDefined<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for VariablesAreDefined<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let variable_names = node
            .variable_definitions
            .as_ref()
            .into_iter()
            .flat_map(|def| def.variable_definitions.iter())
            .map(|def| def.variable.name.as_ref())
            .collect();

        node.traverse(
            &VariablesAreDefinedInOperation {
                database: self.0,
                variable_names,
            },
            accumulator,
        )
    }
}

pub struct VariablesAreDefinedInOperation<'a, T>
where
    T: Eq + Hash,
{
    database: &'a Database<T>,
    variable_names: HashSet<&'a T>,
}

impl<'a, T> Visit<'a, T> for VariablesAreDefinedInOperation<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable(&self, node: &'a Variable<T>, accumulator: &mut Self::Accumulator) {
        if self.variable_names.contains(node.name.as_ref()) {
            return;
        }

        accumulator.push(Diagnostic::undefined_variable(
            node.name.as_ref().to_string(),
            node.span(),
        ));
    }

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(definition) = self
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next() else {
            return
        };

        definition.traverse(
            &VariablesAreDefinedInFragment {
                database: self.database,
                variable_names: &self.variable_names,
                fragment_name: node.fragment_name.as_ref(),
                fragment_span: node.fragment_name.span(),
                stack: vec![node.fragment_name.as_ref()].into_iter().collect(),
            },
            accumulator,
        )
    }
}

pub struct VariablesAreDefinedInFragment<'a, T>
where
    T: Eq + Hash,
{
    database: &'a Database<T>,
    variable_names: &'a HashSet<&'a T>,
    fragment_name: &'a T,
    fragment_span: Span,
    stack: HashSet<&'a T>,
}

impl<'a, T> Visit<'a, T> for VariablesAreDefinedInFragment<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable(&self, node: &'a Variable<T>, accumulator: &mut Self::Accumulator) {
        if self.variable_names.contains(node.name.as_ref()) {
            return;
        }

        accumulator.push(Diagnostic::undefined_variable_in_fragment(
            self.fragment_name.to_string(),
            node.name.as_ref().to_string(),
            self.fragment_span,
            node.span(),
        ));
    }

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self.stack.contains(node.fragment_name.as_ref()) {
            return;
        }

        let Some(definition) = self
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next() else {
            return
        };

        definition.traverse(
            &VariablesAreDefinedInFragment {
                stack: self
                    .stack
                    .iter()
                    .copied()
                    .chain(once(node.fragment_name.as_ref()))
                    .collect(),
                ..*self
            },
            accumulator,
        )
    }
}

use std::collections::HashSet;
use std::hash::Hash;
use std::iter::once;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariablesAreDefined<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for VariablesAreDefined<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'ast Shared<'a, T, OperationDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let variable_names = node
            .variable_definitions
            .as_ref()
            .into_iter()
            .flat_map(|def| def.variable_definitions.iter())
            .filter_map(|def| Some(def.variable.name.ok()?.as_ref()))
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

pub struct VariablesAreDefinedInOperation<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    database: &'ast Database<'a, T>,
    variable_names: HashSet<&'ast T>,
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for VariablesAreDefinedInOperation<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable(&self, node: &'ast Variable<'a, T>, accumulator: &mut Self::Accumulator) {
        let Some(name) = node.name.ok() else { return };

        if self.variable_names.contains(name.as_ref()) {
            return;
        }

        accumulator.push(Diagnostic::undefined_variable(
            name.as_ref().to_string(),
            node.span(),
        ));
    }

    fn visit_fragment_spread(
        &self,
        node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(definition) = self
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next()
        else {
            return;
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

pub struct VariablesAreDefinedInFragment<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    database: &'ast Database<'a, T>,
    variable_names: &'ast HashSet<&'ast T>,
    fragment_name: &'ast T,
    fragment_span: Span,
    stack: HashSet<&'ast T>,
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for VariablesAreDefinedInFragment<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable(&self, node: &'ast Variable<'a, T>, accumulator: &mut Self::Accumulator) {
        let Some(name) = node.name.ok() else { return };

        if self.variable_names.contains(name.as_ref()) {
            return;
        }

        accumulator.push(Diagnostic::undefined_variable_in_fragment(
            self.fragment_name.to_string(),
            name.as_ref().to_string(),
            self.fragment_span,
            node.span(),
        ));
    }

    fn visit_fragment_spread(
        &self,
        node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self.stack.contains(node.fragment_name.as_ref()) {
            return;
        }

        let Some(definition) = self
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next()
        else {
            return;
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

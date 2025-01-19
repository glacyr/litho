use std::collections::HashSet;
use std::hash::Hash;
use std::iter::once;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariablesAreUsed<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for VariablesAreUsed<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'ast Shared<'a, T, OperationDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut used = HashSet::new();

        node.selection_set
            .traverse(&VariableUsage(self.0, HashSet::new()), &mut used);

        for def in node
            .variable_definitions
            .iter()
            .flat_map(|def| def.variable_definitions.iter())
        {
            let Some(name) = def.variable.name.ok() else {
                continue;
            };

            if used.contains(name.as_ref()) {
                continue;
            }

            accumulator.push(Diagnostic::unused_variable(
                name.as_ref().to_string(),
                def.variable.span(),
            ));
        }
    }
}

pub struct VariableUsage<'ast, 'a, T>(&'ast Database<'a, T>, HashSet<&'ast T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for VariableUsage<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = HashSet<&'ast T>;

    fn visit_variable(&self, node: &'ast Variable<'a, T>, accumulator: &mut Self::Accumulator) {
        let Some(name) = node.name.ok() else { return };

        accumulator.insert(name.as_ref());
    }

    fn visit_fragment_spread(
        &self,
        node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self.1.contains(node.fragment_name.as_ref()) {
            return;
        }

        let Some(definition) = self.0.fragments.by_name(node.fragment_name.as_ref()).next() else {
            return;
        };

        definition.traverse(
            &VariableUsage(
                self.0,
                self.1
                    .iter()
                    .copied()
                    .chain(once(node.fragment_name.as_ref()))
                    .collect(),
            ),
            accumulator,
        )
    }
}

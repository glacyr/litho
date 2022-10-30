use std::collections::HashSet;
use std::hash::Hash;
use std::iter::once;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariablesAreUsed<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for VariablesAreUsed<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
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
            if used.contains(def.variable.name.as_ref()) {
                continue;
            }

            accumulator.push(Diagnostic::unused_variable(
                def.variable.name.as_ref().to_string(),
                def.variable.span(),
            ));
        }
    }
}

pub struct VariableUsage<'a, T>(&'a Database<T>, HashSet<&'a T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for VariableUsage<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = HashSet<&'a T>;

    fn visit_variable(&self, node: &'a Variable<T>, accumulator: &mut Self::Accumulator) {
        accumulator.insert(node.name.as_ref());
    }

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self.1.contains(node.fragment_name.as_ref()) {
            return;
        }

        let definition = match self.0.fragments.by_name(node.fragment_name.as_ref()).next() {
            Some(definition) => definition,
            None => return,
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

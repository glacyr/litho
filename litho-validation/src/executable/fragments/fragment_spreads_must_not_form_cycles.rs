use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentSpreadsMustNotFormCycles<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FragmentSpreadsMustNotFormCycles<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'a Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.fragment_name.ok() {
            Some(name) => name.as_ref(),
            None => return,
        };

        node.traverse(
            &DetectFragmentCycles(self.0),
            &mut State {
                name,
                stack: vec![],
                diagnostics: accumulator,
            },
        )
    }
}

pub struct State<'a, T>
where
    T: Eq + Hash,
{
    name: &'a T,
    stack: Vec<(&'a T, &'a FragmentSpread<T>)>,
    diagnostics: &'a mut Vec<Diagnostic<Span>>,
}

pub struct DetectFragmentCycles<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for DetectFragmentCycles<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = State<'a, T>;

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.stack.push((node.fragment_name.as_ref(), node));

        if let Some((name, first)) = accumulator
            .stack
            .iter()
            .take(accumulator.stack.len() - 1)
            .find(|def| def.0 == node.fragment_name.as_ref())
        {
            accumulator
                .diagnostics
                .push(Diagnostic::cyclic_fragment_definition(
                    accumulator.name.to_string(),
                    name.to_string(),
                    first.fragment_name.span(),
                ));

            return;
        };

        if let Some(definition) = self.0.fragments.by_name(node.fragment_name.as_ref()).next() {
            definition.traverse(self, accumulator)
        }
    }

    fn post_visit_fragment_spread(
        &self,
        _node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.stack.pop();
    }
}

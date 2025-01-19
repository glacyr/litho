use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentSpreadsMustNotFormCycles<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for FragmentSpreadsMustNotFormCycles<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'ast Shared<'a, T, FragmentDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.fragment_name.ok().map(AsRef::as_ref) else {
            return;
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

pub struct State<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    name: &'ast T,
    stack: Vec<(&'ast T, &'ast FragmentSpread<'a, T>)>,
    diagnostics: &'ast mut Vec<Diagnostic<Span>>,
}

pub struct DetectFragmentCycles<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for DetectFragmentCycles<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = State<'ast, 'a, T>;

    fn visit_fragment_spread(
        &self,
        node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
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
        _node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.stack.pop();
    }
}

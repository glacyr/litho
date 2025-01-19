use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentSpreadTargetDefined<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for FragmentSpreadTargetDefined<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_spread(
        &self,
        node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self
            .0
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next()
            .is_none()
        {
            accumulator.push(Diagnostic::undefined_fragment(
                node.fragment_name.as_ref().to_string(),
                node.fragment_name.span(),
            ));
        }
    }
}

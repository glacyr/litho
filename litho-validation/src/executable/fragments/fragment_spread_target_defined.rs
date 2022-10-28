use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentSpreadTargetDefined<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FragmentSpreadTargetDefined<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
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

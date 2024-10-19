use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentsMustBeUsed<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FragmentsMustBeUsed<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'a Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.fragment_name.ok() else {
            return;
        };

        if self.0.usages.fragments.usages(node).next().is_none() {
            accumulator.push(Diagnostic::unused_fragment_definition(
                name.as_ref().to_string(),
                name.span(),
            ));
        }
    }
}

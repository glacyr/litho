use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentNameUniqueness<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FragmentNameUniqueness<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'a Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.fragment_name.ok() {
            Some(name) => match self.0.fragments.by_name(name.as_ref()).next() {
                Some(first) if !Arc::ptr_eq(first, node) => {
                    accumulator.push(Diagnostic::duplicate_fragment_name(
                        name.as_ref().to_string(),
                        first.fragment_name.span(),
                        node.fragment_name.span(),
                    ))
                }
                _ => {}
            },
            None => {}
        }
    }
}

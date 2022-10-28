use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentNames<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ArgumentNames<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_argument(&self, node: &'a Arc<Argument<T>>, accumulator: &mut Self::Accumulator) {
        if self
            .0
            .inference
            .definitions_for_arguments
            .get(node)
            .is_none()
        {
            accumulator.push(Diagnostic::undefined_argument(
                node.name.as_ref().to_string(),
                node.name.span(),
            ))
        }
    }
}

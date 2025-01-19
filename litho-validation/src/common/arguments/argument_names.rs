use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentNames<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for ArgumentNames<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_argument(
        &self,
        node: &'ast Shared<'a, T, Argument<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
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

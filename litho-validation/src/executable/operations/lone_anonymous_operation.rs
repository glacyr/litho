use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct LoneAnonymousOperation<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for LoneAnonymousOperation<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.name.is_none() && self.0.operations.len() > 1 {
            accumulator.push(Diagnostic::lone_anonymous_operation(node.span()));
        }
    }
}

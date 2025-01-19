use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct LoneAnonymousOperation<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for LoneAnonymousOperation<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'ast Shared<'a, T, OperationDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.name.is_none() && self.0.operations.len() > 1 {
            accumulator.push(Diagnostic::lone_anonymous_operation(node.span()));
        }
    }
}

use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct OperationNameUniqueness<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for OperationNameUniqueness<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.name.ok() {
            Some(name) => match self.0.operations.by_name(name.as_ref()).next() {
                Some(first) if !Arc::ptr_eq(first, node) => {
                    accumulator.push(Diagnostic::duplicate_operation_name(
                        name.as_ref().to_string(),
                        first.name.span(),
                        node.name.span(),
                    ))
                }
                _ => {}
            },
            None => {}
        }
    }
}

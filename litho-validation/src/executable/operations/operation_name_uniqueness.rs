use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct OperationNameUniqueness<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for OperationNameUniqueness<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'ast Shared<'a, T, OperationDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.as_ref() else {
            return;
        };

        match self.0.operations.by_name(name.as_ref()).next() {
            Some(first) if !Shared::ptr_eq(first, node) => {
                accumulator.push(Diagnostic::duplicate_operation_name(
                    name.as_ref().to_string(),
                    first.name.span(),
                    node.name.span(),
                ))
            }
            _ => {}
        }
    }
}

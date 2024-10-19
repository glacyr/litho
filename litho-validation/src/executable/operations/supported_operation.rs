use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct SupportedOperation<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for SupportedOperation<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(selection_set) = node.selection_set.ok() else {
            return;
        };

        let Some(ty) = self.0.inference.type_by_selection_set.get(selection_set) else {
            return;
        };

        if self.0.type_exists(ty) {
            return;
        }

        let name = match node.ty.as_ref() {
            Some(OperationType::Query(_)) | None => "Query",
            Some(OperationType::Mutation(_)) => "Mutation",
            Some(OperationType::Subscription(_)) => "Subscription",
        };

        accumulator.push(Diagnostic::unsupported_operation(
            name.to_owned(),
            node.ty
                .as_ref()
                .map(|ty| ty.span())
                .unwrap_or(selection_set.span()),
        ))
    }
}

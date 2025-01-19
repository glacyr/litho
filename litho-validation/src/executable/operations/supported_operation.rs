use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct SupportedOperation<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for SupportedOperation<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'ast Shared<'a, T, OperationDefinition<'a, T>>,
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

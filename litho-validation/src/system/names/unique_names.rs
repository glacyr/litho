use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct UniqueNames<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for UniqueNames<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_directive_definition(
        &self,
        node: &'a Arc<DirectiveDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        let Some(first) = self.0.directive_definitions_by_name(name.as_ref()).next() else {
            return;
        };

        if Arc::ptr_eq(first, node) {
            return;
        }

        accumulator.push(Diagnostic::duplicate_directive_name(
            name.as_ref().to_string(),
            first.name.span(),
            node.name.span(),
        ));
    }

    fn visit_type_definition(
        &self,
        node: &'a Arc<TypeDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name().ok() else { return };

        let Some(first) = self.0.type_definitions_by_name(name.as_ref()).next() else {
            return;
        };

        if Arc::ptr_eq(first, node) {
            return;
        }

        accumulator.push(Diagnostic::duplicate_type_name(
            name.as_ref().to_string(),
            first.name().span(),
            node.name().span(),
        ));
    }
}

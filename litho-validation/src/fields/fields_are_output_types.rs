use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FieldsAreOutputTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FieldsAreOutputTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_field_definition(
        &self,
        node: &'a Arc<FieldDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.ty.ok().and_then(|ty| ty.named_type()) {
            Some(name) if !self.0.is_output_type(name.0.as_ref()) => {
                accumulator.push(Diagnostic::field_not_output_type(
                    node.name.as_ref().to_string(),
                    name.0.as_ref().to_string(),
                    name.span(),
                ));
            }
            _ => {}
        }
    }
}

use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct LeafFieldSelections<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for LeafFieldSelections<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_field(&self, node: &'a Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        let Some(name) = self
            .0
            .inference
            .type_for_field(node)
            .and_then(|ty| ty.name()) else {
            return
        };

        let Some(ty) = self.0.type_definitions_by_name(name).next() else {
            return
        };

        let Some(field_name) = node.name.ok() else {
            return
        };

        let diagnostic = match (!ty.is_scalar_like(), node.selection_set.is_some()) {
            (true, true) | (false, false) => return,
            (true, false) => Diagnostic::missing_selection_set(
                field_name.as_ref().to_string(),
                name.to_string(),
                field_name.span(),
            ),
            (false, true) => Diagnostic::unexpected_selection_set(
                field_name.as_ref().to_string(),
                name.to_string(),
                field_name.span(),
            ),
        };

        accumulator.push(diagnostic);
    }
}

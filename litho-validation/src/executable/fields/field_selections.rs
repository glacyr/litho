use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FieldSelections<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for FieldSelections<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_selection_set(
        &self,
        node: &'ast Shared<'a, T, SelectionSet<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(ty) = self.0.inference.type_by_selection_set.get(node) else {
            return;
        };

        if !self.0.is_composite_type(ty) {
            return;
        }

        for selection in node.selections.iter() {
            match selection {
                Selection::Field(field) => {
                    if self
                        .0
                        .inference
                        .field_definitions_by_field
                        .get(field)
                        .is_none()
                    {
                        if let Some(name) = field.name.ok() {
                            accumulator.push(Diagnostic::undefined_field(
                                ty.to_string(),
                                name.as_ref().to_string(),
                                name.span(),
                            ))
                        }
                    }
                }
                Selection::FragmentSpread(_) | Selection::InlineFragment(_) => {}
            }
        }
    }
}

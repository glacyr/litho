use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentOnCompositeTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FragmentOnCompositeTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'a Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.fragment_name.ok() {
            Some(name) => name.as_ref(),
            None => return,
        };

        let ty = match node
            .type_condition
            .ok()
            .and_then(|cond| cond.named_type.ok())
        {
            Some(ty) => ty,
            None => return,
        };

        match self.0.type_definitions_by_name(ty.0.as_ref()).next() {
            Some(def) if !def.is_composite() => {
                accumulator.push(Diagnostic::fragment_on_non_composite_type(
                    name.to_string(),
                    ty.0.as_ref().to_string(),
                    ty.span(),
                ));
            }
            Some(_) | None => {}
        }
    }
}

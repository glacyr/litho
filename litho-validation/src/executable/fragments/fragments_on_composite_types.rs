use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentOnCompositeTypes<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for FragmentOnCompositeTypes<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'ast Shared<'a, T, FragmentDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.fragment_name.ok().map(AsRef::as_ref) else {
            return;
        };

        let Some(ty) = node
            .type_condition
            .ok()
            .and_then(|cond| cond.named_type.ok())
        else {
            return;
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

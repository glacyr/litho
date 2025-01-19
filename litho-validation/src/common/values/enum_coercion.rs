use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct EnumCoercion<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for EnumCoercion<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString + Borrow<str>,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_value(
        &self,
        node: &'ast Shared<'a, T, Value<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.is_variable() {
            return;
        }

        let Some(ty) = self.0.inference.types_for_values.get(&node) else {
            return;
        };

        if matches!(ty.as_ref(), Type::List(_)) {
            return;
        }

        match ty.name() {
            Some(name) if <T as Borrow<str>>::borrow(name) == "Boolean" => return,
            _ => {}
        }

        let definition = ty
            .name()
            .and_then(|name| self.0.type_definitions_by_name(name).next());

        match definition.map(AsRef::as_ref) {
            Some(TypeDefinition::EnumTypeDefinition(_)) => {}
            Some(_) | None => return,
        };

        let value = match node.as_ref() {
            Value::EnumValue(value) => value.0.as_ref(),
            _ => {
                accumulator.push(Diagnostic::expected_enum_value(ty.to_string(), node.span()));
                return;
            }
        };

        let Some(ty) = ty.name() else { return };

        if self
            .0
            .enum_value_definitions_by_name(ty, value)
            .next()
            .is_none()
        {
            accumulator.push(Diagnostic::unrecognized_enum_value(
                ty.to_string(),
                value.to_string(),
                node.span(),
            ))
        }
    }
}

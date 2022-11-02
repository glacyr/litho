use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct EnumCoercion<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for EnumCoercion<'a, T>
where
    T: Eq + Hash + ToString + Borrow<str>,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_value(&self, node: &'a Arc<Value<T>>, accumulator: &mut Self::Accumulator) {
        if node.is_variable() {
            return;
        }

        let ty = match self.0.inference.types_for_values.get(&node) {
            Some(ty) => ty,
            None => return,
        };

        if matches!(ty.as_ref(), Type::List(_)) {
            return;
        }

        match ty.name() {
            Some(name) if name.borrow() == "Boolean" => return,
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

        let ty = match ty.name() {
            Some(name) => name,
            None => return,
        };

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

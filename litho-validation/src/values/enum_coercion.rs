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
        let ty = match self.0.inference.types_for_values.get(&node) {
            Some(ty) => ty,
            None => return,
        };

        let value = match node.as_ref() {
            Value::EnumValue(value) => value.0.as_ref(),
            _ => return,
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

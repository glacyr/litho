use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct InputCoercion<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for InputCoercion<'a, T>
where
    T: Eq + Hash + ToString + Borrow<str>,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_value(&self, node: &'a Arc<Value<T>>, accumulator: &mut Self::Accumulator) {
        let ty = match self.0.inference.types_for_values.get(&node) {
            Some(ty) => ty,
            None => return,
        };

        let ty = match ty.as_ref() {
            Type::NonNull(_) if node.is_null() => {
                accumulator.push(Diagnostic::expected_non_null_value(
                    ty.to_string(),
                    node.span(),
                ));
                return;
            }
            Type::NonNull(ty) => ty.ty.as_ref(),
            _ if node.is_null() => return,
            ty => ty,
        };

        let name = match ty {
            Type::List(_) if !node.is_list() => {
                accumulator.push(Diagnostic::expected_list_value(ty.to_string(), node.span()));
                return;
            }
            Type::Named(named) => named.0.as_ref(),
            _ => return,
        };

        match name.borrow() {
            "Int" if !node.is_int() => {
                accumulator.push(Diagnostic::expected_int_value(ty.to_string(), node.span()));
                return;
            }
            "Float" if !node.is_float_like() => {
                accumulator.push(Diagnostic::expected_float_value(
                    ty.to_string(),
                    node.span(),
                ));
                return;
            }
            "String" if !node.is_string() => {
                accumulator.push(Diagnostic::expected_string_value(
                    ty.to_string(),
                    node.span(),
                ));
                return;
            }
            "Boolean" if !node.is_boolean() => {
                accumulator.push(Diagnostic::expected_boolean_value(
                    ty.to_string(),
                    node.span(),
                ));
                return;
            }
            "ID" if !node.is_id_like() => {
                accumulator.push(Diagnostic::expected_string_value(
                    ty.to_string(),
                    node.span(),
                ));
                return;
            }
            _ => {}
        }
    }
}

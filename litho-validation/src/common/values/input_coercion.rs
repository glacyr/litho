use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct InputCoercion<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> InputCoercion<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString + Borrow<str>,
{
    fn check_ty(&self, ty: &Type<'a, T>, node: &Value<'a, T>) -> Option<Diagnostic<Span>> {
        if node.is_variable() {
            return None;
        }

        let ty = match ty {
            Type::NonNull(_) if node.is_null() => {
                return Some(Diagnostic::expected_non_null_value(
                    ty.to_string(),
                    node.span(),
                ));
            }
            Type::NonNull(ty) => ty.ty.as_ref(),
            _ if node.is_null() => return None,
            ty => ty,
        };

        let name = match ty {
            Type::List(_) if !node.is_list() => {
                return Some(Diagnostic::expected_list_value(ty.to_string(), node.span()));
            }
            Type::List(_) => return None,
            Type::Named(named) => named.0.as_ref(),
            _ => return None,
        };

        match name.borrow() {
            "Int" if !node.is_int() => {
                Some(Diagnostic::expected_int_value(ty.to_string(), node.span()))
            }
            "Float" if !node.is_float_like() => Some(Diagnostic::expected_float_value(
                ty.to_string(),
                node.span(),
            )),
            "String" if !node.is_string() => Some(Diagnostic::expected_string_value(
                ty.to_string(),
                node.span(),
            )),
            "Boolean" if !node.is_boolean() => Some(Diagnostic::expected_boolean_value(
                ty.to_string(),
                node.span(),
            )),
            "ID" if !node.is_id_like() => Some(Diagnostic::expected_string_value(
                ty.to_string(),
                node.span(),
            )),
            _ => None,
        }
    }
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for InputCoercion<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString + Borrow<str>,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_value(
        &self,
        node: &'ast Shared<'a, T, Value<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(ty) = self.0.inference.types_for_values.get(&node) else {
            return;
        };

        accumulator.extend(self.check_ty(&ty, node))
    }
}

use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ObjectCoercion<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ObjectCoercion<'a, T>
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

        let ty = match ty.as_ref() {
            Type::NonNull(ty) => ty.ty.as_ref(),
            ty => ty,
        };

        let name = match ty {
            Type::Named(named) => named.0.as_ref(),
            _ => return,
        };

        for field in self.0.input_value_definitions(name) {
            let object = match node.as_ref() {
                Value::ObjectValue(value) => value,
                _ => {
                    accumulator.push(Diagnostic::expected_input_object_value(
                        ty.to_string(),
                        node.span(),
                    ));
                    return;
                }
            };

            let ty = match field.ty.ok() {
                Some(ty) => ty,
                None => continue,
            };

            if !ty.is_required() {
                continue;
            }

            let value = object
                .object_fields
                .iter()
                .find(|value| value.name.as_ref() == field.name.as_ref());

            if value.is_none() {
                if let Some(brace) = object.braces.1.ok() {
                    accumulator.push(Diagnostic::missing_input_field(
                        field.name.as_ref().to_string(),
                        ty.to_string(),
                        brace.span().collapse_to_start(),
                    ))
                }
            }
        }

        if let Value::ObjectValue(value) = node.as_ref() {
            if let Some(name) = ty.name() {
                for value in value.object_fields.iter() {
                    if self
                        .0
                        .input_value_definitions_by_name(name, value.name.as_ref())
                        .next()
                        .is_none()
                    {
                        accumulator.push(Diagnostic::unrecognized_input_field(
                            value.name.as_ref().to_string(),
                            ty.to_string(),
                            value.name.span(),
                        ))
                    }
                }
            }
        }
    }
}

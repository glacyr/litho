use std::hash::Hash;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct ObjectHasFields<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ObjectHasFields<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            match node.fields_definition.as_ref() {
                Some(def) if def.definitions.is_empty() => {
                    accumulator.push(Error::EmptyObjectType {
                        name: name.as_ref(),
                        span: node
                            .fields_definition
                            .as_ref()
                            .map(|def| def.braces.span())
                            .unwrap_or(name.span()),
                    })
                }
                Some(_) => {}
                None => accumulator.push(Error::MissingFieldsDefinition {
                    name: name.as_ref(),
                    span: node.ty.span(),
                }),
            }
        }
    }
}

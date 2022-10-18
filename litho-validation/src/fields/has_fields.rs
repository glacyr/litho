use std::hash::Hash;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct HasFields<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> HasFields<'a, T>
where
    T: Eq + Hash,
{
    pub fn check_fields_definition(
        &self,
        name: &'a Name<T>,
        definition: Option<&FieldsDefinition<T>>,
    ) -> Option<Error<'a, T>> {
        match definition.as_ref() {
            Some(def) if def.definitions.is_empty() => Some(Error::EmptyType {
                name: name.as_ref(),
                span: definition
                    .as_ref()
                    .map(|def| def.braces.span())
                    .unwrap_or(name.span()),
            }),
            Some(_) => None,
            None => Some(Error::MissingFieldsDefinition {
                name: name.as_ref(),
                span: name.span(),
            }),
        }
    }
}

impl<'a, T> Visit<'a, T> for HasFields<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_interface_type_definition(
        &self,
        node: &'a InterfaceTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            accumulator.extend(
                self.check_fields_definition(name, node.fields_definition.as_ref())
                    .into_iter(),
            )
        }
    }

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            accumulator.extend(
                self.check_fields_definition(name, node.fields_definition.as_ref())
                    .into_iter(),
            );
        }
    }
}

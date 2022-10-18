use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct FieldNameUniqueness<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FieldNameUniqueness<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_input_fields_definition(
        &self,
        node: &'a InputFieldsDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut existing = HashMap::<&T, &InputValueDefinition<T>>::new();

        for field in node.definitions.iter() {
            match existing.get(&field.name.as_ref()) {
                Some(first) => accumulator.push(Error::DuplicateFieldName {
                    name: field.name.as_ref(),
                    first: first.name.span(),
                    second: field.name.span(),
                }),
                None => {
                    existing.insert(field.name.as_ref(), field);
                }
            }
        }
    }

    fn visit_fields_definition(
        &self,
        node: &'a FieldsDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut existing = HashMap::<&T, &Arc<FieldDefinition<T>>>::new();

        for field in node.definitions.iter() {
            match existing.get(&field.name.as_ref()) {
                Some(first) => accumulator.push(Error::DuplicateFieldName {
                    name: field.name.as_ref(),
                    first: first.name.span(),
                    second: field.name.span(),
                }),
                None => {
                    existing.insert(field.name.as_ref(), field);
                }
            }
        }
    }

    fn visit_object_type_extension(
        &self,
        node: &'a ObjectTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        for field in node
            .fields_definition
            .iter()
            .flat_map(|def| def.definitions.iter())
        {
            match self
                .0
                .field_definitions_by_name(name.as_ref(), field.name.as_ref())
                .next()
            {
                Some(first) if !Arc::ptr_eq(first, field) => {
                    accumulator.push(Error::DuplicateFieldName {
                        name: field.name.as_ref(),
                        first: first.name.span(),
                        second: field.name.span(),
                    })
                }
                _ => {}
            }
        }
    }
}

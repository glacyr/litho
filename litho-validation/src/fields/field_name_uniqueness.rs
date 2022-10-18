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
}

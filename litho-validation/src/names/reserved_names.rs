use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct ReservedNames<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ReservedNames<'a, T>
where
    T: Eq + Hash + Borrow<str>,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_input_value_definition(
        &self,
        node: &'a InputValueDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.name.as_ref().borrow().starts_with("__") {
            accumulator.push(Error::ReservedInputValueName {
                name: node.name.as_ref(),
                span: node.name.span(),
            })
        }
    }

    fn visit_field_definition(
        &self,
        node: &'a Arc<FieldDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.name.as_ref().borrow().starts_with("__") {
            accumulator.push(Error::ReservedFieldName {
                name: node.name.as_ref(),
                span: node.name.span(),
            })
        }
    }
}

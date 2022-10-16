use std::hash::Hash;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct NamedTypesExist<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for NamedTypesExist<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_named_type(&self, node: &'a NamedType<T>, accumulator: &mut Self::Accumulator) {
        if self
            .0
            .type_definitions_by_name(node.0.as_ref())
            .next()
            .is_none()
        {
            accumulator.push(Error::UnknownNamedType {
                name: node.0.as_ref(),
                span: node.span(),
            })
        }
    }
}

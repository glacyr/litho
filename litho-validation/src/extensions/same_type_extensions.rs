use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct SameTypeExtensions<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for SameTypeExtensions<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_type_extension(
        &self,
        node: &'a Arc<TypeExtension<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name().ok() {
            Some(name) => name,
            None => return,
        };

        let first = match self.0.type_definitions_by_name(name.as_ref()).next() {
            Some(first) => first.keyword(),
            None => return,
        };

        let second = node.keyword();

        if first.as_ref() != second.as_ref() {
            accumulator.push(Error::DifferentExtensionType {
                name: name.as_ref(),
                first: first.span(),
                first_type: first.as_ref(),
                second: second.span(),
                second_type: second.as_ref(),
            })
        }
    }
}

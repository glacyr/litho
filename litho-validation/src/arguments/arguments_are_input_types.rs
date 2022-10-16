use std::hash::Hash;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct ArgumentsAreInputTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ArgumentsAreInputTypes<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_input_value_definition(
        &self,
        node: &'a InputValueDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.ty.ok().and_then(|ty| ty.named_type()) {
            Some(name) if !self.0.is_input_type(name.0.as_ref()) => {
                accumulator.push(Error::InputValueNotInputType {
                    name: name.0.as_ref(),
                    span: name.span(),
                })
            }
            _ => {}
        }
    }
}

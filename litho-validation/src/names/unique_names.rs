use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct UniqueNames<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for UniqueNames<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_directive_definition(
        &self,
        node: &'a Arc<DirectiveDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        let first = match self.0.directive_definitions_by_name(name.as_ref()).next() {
            Some(first) => first,
            None => return,
        };

        if Arc::ptr_eq(first, node) {
            return;
        }

        accumulator.push(Error::DuplicateDirectiveName {
            name: name.as_ref(),
            first: first.name.span(),
            second: node.name.span(),
        })
    }

    fn visit_type_definition(
        &self,
        node: &'a Arc<TypeDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name().ok() {
            Some(name) => name,
            None => return,
        };

        let first = match self.0.type_definitions_by_name(name.as_ref()).next() {
            Some(first) => first,
            None => return,
        };

        if Arc::ptr_eq(first, node) {
            return;
        }

        accumulator.push(Error::DuplicateTypeName {
            name: name.as_ref(),
            first: first.name().span(),
            second: node.name().span(),
        })
    }
}

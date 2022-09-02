use graphql_parser::query::{Definition, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.2.2.1 Lone Anonymous Operation
///
/// ## Formal Specification
/// - Let `operations` be all operation definitions in the document.
/// - Let `anonymous` be all anonymous operation definitions in the document.
/// - If `operations` is a set of more than 1:
///   - `anonymous` must be empty.
///
/// ## Explanatory Text
/// GraphQL allows a short-hand form for defining query operations when only
/// that one operation exists in the document.
///
/// For example the following document is valid:
/// ```graphql
/// {
///   dog {
///     name
///   }
/// }
/// ```
///
/// While this document is invalid:
/// ```graphql
/// {
///   dog {
///     name
///   }
/// }
///
/// query getName {
///   dog {
///     owner {
///       name
///     }
///   }
/// }
/// ```
pub struct LoneAnonymousOperation;

impl<'v, 'a, T> Visitor<'v, 'a, T> for LoneAnonymousOperation
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_document(
        &self,
        document: &'v query::Document<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        _scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
        let operations = document
            .definitions
            .iter()
            .flat_map(|definition| match definition {
                Definition::Operation(operation) => Some(operation),
                _ => None,
            });

        let mut anonymous = operations
            .clone()
            .filter(|operation| operation.name().is_none());

        match anonymous.next() {
            Some(_) if operations.clone().count() > 1 => {
                let (anonymous, named) =
                    operations.partition(|operation| operation.name().is_none());

                accumulator.push(Error::MixedAnonymousOperation { anonymous, named })
            }
            _ => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for LoneAnonymousOperation
where
    'a: 'v,
    T: Text<'a>,
{
}

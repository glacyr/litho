use std::collections::HashMap;

use graphql_parser::query::{Definition, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.2.1.1 Operation Name Uniqueness
///
/// ## Formal Specification
/// - For each operation definition `operation` in the document.
/// - Let `operationName` be the name of `operation`.
/// - If `operationName` exists:
///   - Let `operations` be all operation definitions in the document named
///     `operationName`.
///   - `operations` must be a set of one.
///
/// ## Explanatory Text
/// Each named operation definition must be unique within a document when
/// referred to by its name.
///
/// For example the following document is valid:
/// ```graphql
/// query getDogName {
///   dog {
///     name
///   }
/// }
///
/// query getOwnerName {
///   dog {
///     owner {
///       name
///     }
///   }
/// }
/// ```
///
/// While this document is invalid:
/// ```graphql
/// query getName {
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
///
/// It is invalid even if the type of each operation is different:
/// ```graphql
/// query dogOperation {
///   dog {
///     name
///   }
/// }
///
/// mutation dogOperation {
///   mutateDog {
///     id
///   }
/// }
/// ```
pub struct OperationNameUniqueness;

impl<'v, 'a, T> Visitor<'v, 'a, T> for OperationNameUniqueness
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
        let mut by_name = HashMap::new();

        for definition in &document.definitions {
            let operation = match definition {
                Definition::Operation(operation) => operation,
                _ => continue,
            };

            let name = match operation.name() {
                Some(name) => name.as_ref(),
                None => continue,
            };

            if let Some(original) = by_name.get(name) {
                accumulator.push(Error::DuplicateOperationName(*original, operation));
            } else {
                by_name.insert(name, operation);
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for OperationNameUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
}

use std::collections::HashMap;

use graphql_parser::query::{Definition, Text};
use graphql_parser::{query, schema};

use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.1.1 Fragment Name Uniqueness
/// ## Formal Specification
/// - For each fragment definition `fragment` in the document
/// - Let `fragmentName` be the name of `fragment`.
/// - Let `fragments` be all fragment definitions in the document named
///   `fragmentName`.
/// - `fragments` must be a set of one.
///
/// ## Explanatory Text
/// Fragment definitions are referenced in fragment spreads by name. To avoid
/// ambiguity, each fragment's name must be unique within a document.
///
/// Inline fragments are not considered fragment definitions, and are unaffected
/// by this rule.
///
/// For example the following document is valid:
/// ```graphql
/// {
///   dog {
///     ...fragmentOne
///     ...fragmentTwo
///   }
/// }
///
/// fragment fragmentOne on Dog {
///   name
/// }
///
/// fragment fragmentTwo on Dog {
///   name
/// }
/// ```
///
/// While this document is invalid:
/// ```graphql
/// {
///   dog {
///     .fragmentOne
///   }
/// }
///
/// fragment fragmentOne on Dog {
///   name
/// }
///
/// fragment fragmentOne on Dog {
///   owner {
///     name
///   }
/// }
/// ```
pub struct FragmentNameUniqueness;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentNameUniqueness
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
        let mut fragments = HashMap::new();

        for definition in &document.definitions {
            let fragment = match definition {
                Definition::Fragment(fragment) => fragment,
                Definition::Operation(_) => continue,
            };

            if let Some(&existing) = fragments.get(fragment.name.as_ref()) {
                accumulator.push(Error::DuplicateFragmentName(existing, fragment));
            } else {
                fragments.insert(fragment.name.as_ref(), fragment);
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentNameUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
}

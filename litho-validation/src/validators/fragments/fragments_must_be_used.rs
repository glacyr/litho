use std::collections::HashSet;

use graphql_parser::query::{Definition, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.1.4 Fragments Must Be Used
/// ## Formal Specification
/// - For each `fragment` defined in the document.
/// - `fragment` must be the target of at least one spread in the document
///
/// ## Explanatory Text
/// Defined fragments must be used within a document.
///
/// For example the following is an invalid document:
/// ```graphql
/// fragment nameFragment on Dog { # unused
///   name
/// }
///
/// {
///   dog {
///     name
///   }
/// }
/// ```
pub struct FragmentsMustBeUsed;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentsMustBeUsed
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_document(
        &self,
        document: &'v query::Document<'a, T>,
        schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut used = HashSet::new();
        FragmentSpreadVisitor.traverse(document, schema, &mut used);

        for operation in document.definitions.iter() {
            let fragment = match operation {
                Definition::Fragment(fragment) if !used.contains(fragment.name.as_ref()) => {
                    fragment
                }
                _ => continue,
            };

            accumulator.push(Error::UnusedFragment {
                fragment_name: &fragment.name,
                span: fragment.span(),
            })
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentsMustBeUsed
where
    'a: 'v,
    T: Text<'a>,
{
}

struct FragmentSpreadVisitor;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadVisitor
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = HashSet<&'v str>;

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v query::FragmentSpread<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.insert(fragment_spread.fragment_name.as_ref());
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadVisitor
where
    'a: 'v,
    T: Text<'a>,
{
}

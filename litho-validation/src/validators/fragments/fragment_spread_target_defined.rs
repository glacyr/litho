use std::collections::HashSet;

use graphql_parser::query::{Definition, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.2.1 Fragment Spread Target Defined
/// ## Formal Specification
/// - For every `namedSpread` in th edocument.
/// - Let `fragment` be the target of `namedSpread`.
/// - `fragment` must be defined in the document.
///
/// ## Explanatory Text
/// Named fragments spreads must refer to fragments defined within the document.
/// It is a validation error if the target of a spread is not defined.
pub struct FragmentSpreadTargetDefined;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadTargetDefined
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
        let fragments = document
            .definitions
            .iter()
            .flat_map(|definition| match definition {
                Definition::Fragment(fragment) => Some(fragment.name.as_ref()),
                Definition::Operation(_) => None,
            })
            .collect();

        let inner = FragmentSpreadTargetDefinedInner { fragments };
        inner.traverse(document, schema, accumulator);
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadTargetDefined
where
    'a: 'v,
    T: Text<'a>,
{
}

struct FragmentSpreadTargetDefinedInner<'v> {
    fragments: HashSet<&'v str>,
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadTargetDefinedInner<'v>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v query::FragmentSpread<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !self
            .fragments
            .contains(fragment_spread.fragment_name.as_ref())
        {
            accumulator.push(Error::UndefinedFragment {
                fragment_name: &fragment_spread.fragment_name,
                span: fragment_spread.span(),
            })
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadTargetDefinedInner<'v>
where
    'a: 'v,
    T: Text<'a>,
{
}

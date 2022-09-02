use std::collections::HashSet;

use graphql_parser::query::{Definition, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.2.2 Fragment Spreads Must Not Form Cycles
/// ## Formal Specification
/// - For each `fragmentDefinition` in the document
/// - Let `visited` be the empty set.
/// - `DetectFragmentCycles(fragmentDefinition, visited):`
///   1. Let `spreads` be all fragment spread descendants of
///      `fragmentDefinition`
///   2. For each `spread` in `spreads`
///      a. `visited` must not contain `spread`
///      b. Let `nextVisited` be the set including `spread` and members of
///         `visited`
///      c. Let `nextFragmentDefinition` be the target of `spread`
///      d. `DetectFragmentCycles(nextFragmentDefinition, nextVisited)`
///
/// ## Explanatory Text
/// The graph of fragment spreads must not form any cycles including spreading
/// itself. Otherwise an operation could infinitely spread or infinitely execute
/// on cycles in the underlying data.
///
/// This invalidates fragments that would result in an infinite spread:
/// ```graphql
/// {
///   dog {
///     ...nameFragment
///   }
/// }
///
/// fragment nameFragment on Dog {
///   name
///   ...barkVolumeFragment
/// }
///
/// fragment barkVolumeFragment on Dog {
///   barkVolume
///   ...nameFragment
/// }
/// ```
///
/// If the above fragments were inlined, this would result in the infinitely
/// large:
/// ```graphql
/// {
///   dog {
///     name
///     barkVolume
///     name
///     barkVolume
///     name
///     barkVolume
///     name
///     # forever...
///   }
/// }
/// ```
///
/// This also invalidates fragments that would result in an infinite recursion
/// when executed against cyclic data.
/// ```graphql
/// {
///   dog {
///     ...dogFragment
///   }
/// }
///
/// fragment dogFragment on Dog {
///   name
///   owner {
///     ...ownerFragment
///   }
/// }
///
/// fragment ownerFragment on Human {
///   name
///   pets [
///     ...dogFragment
///   ]
/// }
/// ```
pub struct FragmentSpreadsMustNotFormCycles;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadsMustNotFormCycles
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_document(
        &self,
        document: &'v query::Document<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        for definition in document.definitions.iter() {
            let fragment = match definition {
                Definition::Fragment(fragment) => fragment,
                Definition::Operation(_) => continue,
            };

            FragmentSpreadsMustNotFormCyclesInner {
                document,
                visited: HashSet::new(),
            }
            .traverse_fragment_definition(fragment, schema, scope, accumulator);
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadsMustNotFormCycles
where
    'a: 'v,
    T: Text<'a>,
{
}

pub struct FragmentSpreadsMustNotFormCyclesInner<'v, 'a, T>
where
    T: Text<'a>,
{
    document: &'v query::Document<'a, T>,
    visited: HashSet<&'v str>,
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadsMustNotFormCyclesInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v query::FragmentSpread<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self
            .visited
            .contains(fragment_spread.fragment_name.as_ref())
        {
            accumulator.push(Error::CyclicFragmentSpread {
                fragment_name: &fragment_spread.fragment_name,
                span: fragment_spread.span(),
            });

            return;
        }

        match self
            .document
            .fragment_definition(fragment_spread.fragment_name.as_ref())
        {
            Some(fragment) => {
                let mut visited = self.visited.to_owned();
                visited.insert(fragment_spread.fragment_name.as_ref());

                FragmentSpreadsMustNotFormCyclesInner {
                    document: self.document,
                    visited,
                }
                .traverse_fragment_definition(fragment, schema, scope, accumulator);
            }
            None => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadsMustNotFormCyclesInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
}

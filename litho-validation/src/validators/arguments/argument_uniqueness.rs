use std::collections::HashSet;

use graphql_parser::query::{Selection, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.4.2 Argument Uniqueness
///
/// Fields and directives treat arguments as a mapping of argument name to
/// value. More than one argument with the same name in an argument set is
/// ambiguous and invalid.
///
/// ## Formal Specification
/// - For each `argument` in the Document.
/// - Let `argumentName` be the Name of `argument`.
/// - Let `arguments` be all Arguments named `argumentName` in the Argument Set
///   which contains `argument`.
/// - `arguments` must be the set containing only `argument`.
pub struct ArgumentUniqueness;

impl<'v, 'a, T> Visitor<'v, 'a, T> for ArgumentUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_selection_set(
        &self,
        selection_set: &'v query::SelectionSet<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
        let ty = match schema.type_definition(&scope.ty()) {
            Some(ty) => ty,
            None => return,
        };

        for selection in &selection_set.items {
            match selection {
                Selection::Field(selection) => {
                    let mut unique = HashSet::new();
                    let mut dups = HashSet::new();

                    for (name, _) in &selection.arguments {
                        if !unique.contains(&name.as_ref()) {
                            unique.insert(name.as_ref());
                        } else {
                            dups.insert(name.as_ref());
                        }
                    }

                    let mut dups = dups.into_iter().collect::<Vec<_>>();
                    dups.sort();

                    accumulator.extend(dups.into_iter().map(|name| Error::DuplicateArgumentName {
                        field_name: &selection.name,
                        field_span: selection.span(),
                        ty: ty.name().to_owned(),
                        name,
                    }));
                }
                Selection::InlineFragment(_) | Selection::FragmentSpread(_) => continue,
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for ArgumentUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
}

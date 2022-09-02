use graphql_parser::query::{Selection, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.3.1 Field Selections
///
/// Field selections must exist on `Object`, `Interface` and `Union` types.
///
/// ## Formal Specification
/// - For each `selection` in the document.
/// - Let `fieldName` be the target field of `selection`.
/// - `fieldName` must be defined on type in scope.
///
/// ## Explanatory Text
/// The target field of a field selection must be defined on the scoped type of
/// the selection set. There are no limitations on alias names.
///
/// For example the following fragment would not pass validation:
/// ```graphql
/// fragment fieldNotDefined on Dog {
///   meowVolume
/// }
///
/// fragment aliasedLyingFieldTargetNotDefined on Dog {
///   barkVolume: kawVolume
/// }
/// ```
///
/// For interfaces, direct field selection can only be done on fields. Fields of
/// concrete implementors are not relevant to the validity of the given
/// interface-typed selection set.
///
/// For example, the following is valid:
/// ```graphql
/// fragment interfaceFieldSelection on Pet {
///   name
/// }
/// ```
///
/// and the following is invalid:
/// ```graphql
/// fragment definedOnImplementorsButNotInterface on Pet {
///   nickname
/// }
/// ```
///
/// Because unions do not define fields, fields may not be directly selected
/// from a union-typed selection set, with the exception of the meta-field
/// `__typename`. Fields from a union-typed selection set must only be queried
/// indirectly via a fragment.
///
/// For example the following is valid:
/// ```graphql
/// fragment inDirectFieldSelectionOnUnion on CatOrDog {
///   __typename
///   ... on Pet {
///     name
///   }
///   ... on Dog {
///     barkVolume
///   }}
/// }
/// ```
///
/// But the following is invalid:
/// ```graphql
/// fragment directFieldSelectionOnUnion on CatOrDog {
///   name
///   barkVolume
/// }
/// ```
pub struct FieldSelections;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FieldSelections
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_selection_set(
        &self,
        selection_set: &'v query::SelectionSet<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let ty = match schema.type_definition(&scope.ty()) {
            Some(ty) => ty,
            None => return,
        };

        for selection in &selection_set.items {
            match selection {
                Selection::Field(selection) => {
                    if ty.field(&selection.name).is_none() {
                        if scope.is_fragment() && !ty.is_composite() {
                            continue;
                        }

                        accumulator.push(Error::UndefinedField {
                            field_name: scope.field_name(),
                            parent_span: scope.span(),
                            ty: ty.name(),
                            field: selection,
                        });
                    };
                }
                Selection::InlineFragment(_) | Selection::FragmentSpread(_) => continue,
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FieldSelections
where
    'a: 'v,
    T: Text<'a>,
{
}

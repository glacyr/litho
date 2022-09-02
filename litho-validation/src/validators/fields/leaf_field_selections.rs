use graphql_parser::query::Text;
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.3.3 Leaf Field Selections
///
/// ## Formal Specification
/// - For each `selection` in the document
/// - Let `selectionType` be the result type of `selection`
/// - If `selectionType` is a scalar or enum:
///   - The subselection set of that selection must be empty
/// - If `selectionType` is an interface, union, or object
///   - The subselection set of that selection must NOT BE empty
///
/// ## Explanatory Text
/// Field selections on scalars or enums are never allowed, because they are
/// leaf nodes of any GraphQL operation.
///
/// The following is valid.
/// ```graphql
/// fragment scalarSelection on Dog {
///   barkVolume
/// }
/// ```
///
/// The following is invalid.
/// ```graphql
/// fragment scalarSelectionsNotAllowedOnInt on Dog {
///   barkVolume {
///     sinceWhen
///   }
/// }
/// ```
///
/// Conversely the leaf field selections of GraphQL operations must be of type
/// scalar or enum. Leaf selections on objects, interfaces, and unions without
/// subfields are disallowed.
///
/// Let's assume the following additions to the query root operation type of the
/// schema:
/// ```graphql
/// extend type Query {
///   human: Human
///   pet: Pet
///   catOrDog: CatOrDog
/// }
/// ```
///
/// The following examples are invalid
/// ```graphql
/// query directQueryAnObjectWithoutSubFields {
///   human
/// }
///
/// query directQueryOnInterfaceWithoutSubFields {
///   pet
/// }
///
/// query directQueryOnUnionWithoutSubFields {
///   catOrDog
/// }
/// ```
pub struct LeafFieldSelections;

impl<'v, 'a, T> Visitor<'v, 'a, T> for LeafFieldSelections
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
        if scope.is_fragment() {
            return;
        }

        let ty = match schema.type_definition(scope.ty()) {
            Some(ty) => ty,
            None => match scope.ty().as_ref() {
                "Int" | "Float" | "String" | "Boolean" | "ID"
                    if !selection_set.items.is_empty() =>
                {
                    accumulator.push(Error::UnexpectedSubselection {
                        field_name: scope.field_name(),
                        parent_span: scope.span(),
                        ty: scope.ty().to_owned(),
                        span: selection_set.items[0].span(),
                    });
                    return;
                }
                _ => return,
            },
        };

        match (ty.is_composite(), !selection_set.items.is_empty()) {
            (true, true) | (false, false) => {}
            (true, false) => {
                accumulator.push(Error::MissingSubselection {
                    field_name: scope.field_name(),
                    parent_span: scope.span(),
                    ty: ty.name(),
                });
            }
            (false, true) => {
                accumulator.push(Error::UnexpectedSubselection {
                    field_name: scope.field_name(),
                    parent_span: scope.span(),
                    ty: scope.ty().to_owned(),
                    span: selection_set.items[0].span(),
                });
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for LeafFieldSelections
where
    'a: 'v,
    T: Text<'a>,
{
}

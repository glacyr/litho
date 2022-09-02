use graphql_parser::query::{Text, TypeCondition};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.1.3 Fragments On Composite Types
/// ## Formal Specification
/// - For each `fragment` defined in the document.
/// - The target type of fragment must have kind `UNION`, `INTERFACE`, or
///   `OBJECT`.
///
/// ## Explanatory Text
/// Fragments can only be declared on unions, interfaces, and objects. They are
/// invalid on scalars. They can only be applied on non-leaf fields. This rule
/// applies to both inline and named fragments.
///
/// The following fragment declarations are valid:
/// ```graphql
/// fragment fragOnObject on Dog {
///   name
/// }
///
/// fragment fragOnInterface on Pet {
///   name
/// }
///
/// fragment fragOnUnion on CatOrDog {
///   ... on Dog {
///     name
///   }
/// }
/// ```
///
/// and the following are invalid:
/// ```graphql
/// fragment fragOnScalar on Int {
///   something
/// }
///
/// fragment inlineFragOnScalar on Dog {
///   ... on Boolean {
///     somethingElse
///   }
/// }
/// ```
pub struct FragmentsOnCompositeTypes;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentsOnCompositeTypes
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_inline_fragment(
        &self,
        inline_fragment: &'v query::InlineFragment<'a, T>,
        schema: &'v schema::Document<'a, T>,
        _scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
        let ty = match inline_fragment.type_condition {
            Some(TypeCondition::On(ref ty)) => ty,
            None => return,
        };

        match ty.as_ref() {
            "Int" | "Float" | "String" | "Boolean" | "ID" => {
                accumulator.push(Error::NonCompositeFragmentTarget {
                    fragment_name: None,
                    span: inline_fragment.span(),
                    name: ty,
                });
                return;
            }
            _ => {}
        }

        match schema.type_definition(ty) {
            Some(definition) if !definition.is_composite() => {
                accumulator.push(Error::NonCompositeFragmentTarget {
                    fragment_name: None,
                    span: inline_fragment.span(),
                    name: ty,
                })
            }
            _ => {}
        }
    }

    fn visit_fragment_definition(
        &self,
        fragment_definition: &'v query::FragmentDefinition<'a, T>,
        schema: &'v schema::Document<'a, T>,
        _scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
        let ty = match fragment_definition.type_condition {
            TypeCondition::On(ref ty) => ty,
        };

        match schema.type_definition(ty) {
            Some(definition) if !definition.is_composite() => {
                accumulator.push(Error::NonCompositeFragmentTarget {
                    fragment_name: Some(&fragment_definition.name),
                    span: fragment_definition.span(),
                    name: ty,
                })
            }
            _ => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentsOnCompositeTypes
where
    'a: 'v,
    T: Text<'a>,
{
}

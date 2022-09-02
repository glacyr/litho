use graphql_parser::query::{Text, TypeCondition};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.2.1 Fragment Spread Type Existence
/// ## Formal Specification
/// - For each named spread `namedSpread` in the document
/// - Let `fragment` be the target of `namedSpread`
/// - The target type of `fragment` must be defined in the schema
///
/// ## Explanatory Text
/// Fragments must be specified on types that exist in the schema. This applies
/// for both named and inline fragments. If they are not defined in the schema,
/// the fragment is invalid.
///
/// For example the following fragments are valid:
/// ```graphql
/// fragment correctType on Dog {
///   name
/// }
///
/// fragment inlineFragment on Dog {
///   ... on Dog {
///     name
///   }
/// }
///
/// fragment inlineFragment2 on Dog {
///   ... @include(if: true) {
///     name
///   }
/// }
/// ```
///
/// and the following do not validate:
/// ```graphql
/// fragment notOnExistingType on NotInSchema {
///   name
/// }
///
/// fragment inlineNotExistingType on Dog {
///   ... on NotInSchema {
///     name
///   }
/// }
/// ```
pub struct FragmentSpreadTypeExistence;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadTypeExistence
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
            "Int" | "Float" | "String" | "Boolean" | "ID" => return,
            _ => {}
        }

        if schema.type_definition(ty).is_none() {
            accumulator.push(Error::UndefinedFragmentTarget {
                fragment_name: None,
                span: inline_fragment.span(),
                name: ty,
            })
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

        if schema.type_definition(ty).is_none() {
            accumulator.push(Error::UndefinedFragmentTarget {
                fragment_name: Some(&fragment_definition.name),
                span: fragment_definition.span(),
                name: ty,
            })
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadTypeExistence
where
    'a: 'v,
    T: Text<'a>,
{
}

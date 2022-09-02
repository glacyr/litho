use std::collections::HashMap;

use graphql_parser::query::{Definition, Text, TypeCondition};
use graphql_parser::{query, schema};

use crate::extensions::*;
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

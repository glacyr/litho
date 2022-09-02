use std::collections::HashSet;

use graphql_parser::query::{Document, Text, TypeCondition};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.5.2.3 Fragment Spread Is Possible
/// ## Formal Specification
/// - For each `spread` (named or inline) defined in the document.
/// - Let `fragment` be the target of `spread`
/// - Let `fragmentType` be the type condition of `fragment`
/// - Let `parentType` be the type of the selection set containing `spread`
/// - Let `applicableTypes` be the intersection of
///   `GetPossibleTypes(fragmentType)` and `GetPossibleTypes(parentType)`
/// - `applicableTypes` must not be empty.
///
/// `GetPossibleTypes(type)`:
/// 1. If `type` is an object type, return a set containing `type`.
/// 2. If `type` is an interface type, return the set of types implementing
///    `type`
/// 3. If `type` is a union type, return the set of possible types of `type`
///
/// ## Explanatory Text
/// Fragments are declared on a type and will only apply when the runtime object
/// type matches the type condition. They also are spread within the context of
/// a parent type. A fragment spread is only valid if its type condition could
/// ever apply within the parent type.
pub struct FragmentSpreadIsPossible;

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadIsPossible
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
        FragmentSpreadIsPossibleInner { document }.traverse(document, schema, accumulator);
    }

    fn visit_inline_fragment(
        &self,
        inline_fragment: &'v query::InlineFragment<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let fragment_type = match &inline_fragment.type_condition {
            Some(TypeCondition::On(ty)) => ty.as_ref(),
            None => return,
        };

        let parent_type = scope.ty();

        let fragment_possible_types = schema.possible_types(fragment_type).collect::<HashSet<_>>();
        let parent_possible_types = schema.possible_types(parent_type).collect::<HashSet<_>>();

        if fragment_possible_types.is_disjoint(&parent_possible_types) {
            accumulator.push(Error::ImpossibleInlineFragment {
                fragment_type,
                fragment_span: inline_fragment.span(),
                parent_type,
                parent_span: scope.span(),
            })
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadIsPossible
where
    'a: 'v,
    T: Text<'a>,
{
}

struct FragmentSpreadIsPossibleInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    document: &'v Document<'a, T>,
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for FragmentSpreadIsPossibleInner<'v, 'a, T>
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
        let fragment = match self
            .document
            .fragment_definition(fragment_spread.fragment_name.as_ref())
        {
            Some(fragment) => fragment,
            None => return,
        };

        let fragment_type = match &fragment.type_condition {
            TypeCondition::On(ty) => ty.as_ref(),
        };

        let parent_type = scope.ty();

        let fragment_possible_types = schema.possible_types(fragment_type).collect::<HashSet<_>>();
        let parent_possible_types = schema.possible_types(parent_type).collect::<HashSet<_>>();

        if fragment_possible_types.is_disjoint(&parent_possible_types) {
            accumulator.push(Error::ImpossibleFragmentSpread {
                fragment_name: fragment.name.as_ref(),
                fragment_type,
                fragment_span: fragment.span(),
                parent_type,
                parent_span: scope.span(),
            })
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FragmentSpreadIsPossibleInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    interface Pet {
        name: String!
    }

    type Dog implements Pet {
        barkVolume: Int!
    }

    type Cat {
        meowVolume: Int!
    }

    union CatOrDog = Cat | Dog

    interface Sentient {
        name: String!
    }

    type Human implements Sentient {
        name: String!
    }

    type Alien implements Sentient {
        name: String!
    }

    union HumanOrAlien = Human | Alien
    union DogOrHuman = Dog | Human

    interface Node {
        id: ID!
    }

    interface Resource implements Node {
        id: ID!
        url: String
    }

    type Query {
        dog: Dog!
        sentient: Sentient!
        humanOrAlien: HumanOrAlien!
        node: Node!
        resource: Resource!
    }
    "#;

    #[test]
    fn test_object_spreads_in_object_scope_148() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment dogFragment on Dog {
            ... on Dog {
                barkVolume
            }
        }

        {
            dog {
                ... dogFragment
            }
        }
        "#,
        )
    }

    #[test]
    fn test_object_spreads_in_object_scope_149() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment catInDogFragmentInvalid on Dog {
            ... on Cat {
                meowVolume
            }
        }

        {
            dog {
                ... catInDogFragmentInvalid
            }
        }
        "#,
            r#"
        Error: 5.5.2.3 Fragment Spread Is Possible

          × Fragment is applied to unrelated type `Cat`.
            ╭────
          1 │ fragment catInDogFragmentInvalid on Dog {
            · ───┬────
            ·    ╰── Fragment is used on selection of type `Dog` here ...
            ·
          2 │     ... on Cat {
            ·         ┬─
            ·         ╰── ... but applies to type `Cat` here.
            ·
          3 │         meowVolume
          4 │     }
          5 │ }
          6 │ 
          7 │ {
          8 │     dog {
          9 │         ... catInDogFragmentInvalid
         10 │     }
         11 │ }
            ╰────
        "#,
        )
    }

    #[test]
    fn test_abstract_spreads_in_object_scope_150() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment petNameFragment on Pet {
            name
        }

        fragment interfaceWithinObjectFragment on Dog {
            ...petNameFragment
        }

        {
            dog {
                ... interfaceWithinObjectFragment
            }
        }
        "#,
        )
    }

    #[test]
    fn test_abstract_spreads_in_object_scope_151() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment catOrDogNameFragment on CatOrDog {
            ... on Cat {
                meowVolume
            }
        }

        fragment unionWithObjectFragment on Dog {
            ...catOrDogNameFragment
        }

        {
            dog {
                ...unionWithObjectFragment
            }
        }
        "#,
        )
    }

    #[test]
    fn test_object_spreads_in_abstract_scope_152() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment petFragment on Pet {
            name
            ... on Dog {
                barkVolume
            }
        }

        fragment catOrDogFragment on CatOrDog {
            ... on Cat {
                meowVolume
            }
        }

        {
            dog {
                ...petFragment
                ...catOrDogFragment
            }
        }
        "#,
        )
    }

    #[test]
    fn test_object_spreads_in_abstract_scope_153() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment sentientFragment on Sentient {
            ... on Dog {
                barkVolume
            }
        }

        {
            sentient {
                ...sentientFragment
            }
        }
        "#,
            r#"
        Error: 5.5.2.3 Fragment Spread Is Possible

          × Fragment is applied to unrelated type `Dog`.
            ╭────
          1 │ fragment sentientFragment on Sentient {
            · ───┬────
            ·    ╰── Fragment is used on selection of type `Sentient` here ...
            ·
          2 │     ... on Dog {
            ·         ┬─
            ·         ╰── ... but applies to type `Dog` here.
            ·
          3 │         barkVolume
          4 │     }
          5 │ }
          6 │ 
          7 │ {
          8 │     sentient {
          9 │         ...sentientFragment
         10 │     }
         11 │ }
            ╰────
        "#,
        )
    }

    #[test]
    fn test_abstract_spreads_in_abstract_scope_154() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment unionWithInterface on Pet {
            ... dogOrHumanFragment
        }

        fragment dogOrHumanFragment on DogOrHuman {
            ... on Dog {
                barkVolume
            }
        }

        {
            dog {
                ... unionWithInterface
            }
        }
        "#,
        )
    }

    #[test]
    fn test_abstract_spreads_in_abstract_scope_155() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment nonIntersectingInterfaces on Pet {
            ...sentientFragment
        }

        fragment sentientFragment on Sentient {
            name
        }

        {
            dog {
                ...nonIntersectingInterfaces
            }
        }
        "#,
            r#"
        Error: 5.5.2.3 Fragment Spread Is Possible

          × Fragment `sentientFragment` can only be applied to type `Sentient`.
            ╭────
          1 │ fragment nonIntersectingInterfaces on Pet {
            · ───┬────
            ·    ╰── ... but is used on selection of type `Pet` here.
            ·
          2 │     ...sentientFragment
          3 │ }
          4 │ 
          5 │ fragment sentientFragment on Sentient {
            · ───┬────
            ·    ╰── Fragment `sentientFragment` can only be applied to type `Sentient` ...
            ·
          6 │     name
          7 │ }
          8 │ 
          9 │ {
         10 │     dog {
         11 │         ...nonIntersectingInterfaces
         12 │     }
         13 │ }
            ╰────
        "#,
        )
    }

    #[test]
    fn test_interface_spreads_in_implemented_interface_scope() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment interfaceWithInterface on Node {
            ... resourceFragment
        }

        fragment resourceFragment on Resource {
            url
        }

        {
            node {
                ... interfaceWithInterface
            }

            resource {
                ... resourceFragment
            }
        }
        "#,
        )
    }
}

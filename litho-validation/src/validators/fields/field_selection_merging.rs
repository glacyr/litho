use std::collections::HashMap;

use graphql_parser::query::{Field, Text, Type};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.3.2 Field Selection Merging
/// ## Formal Specification
/// - Let `set` be any selection set defined in the GraphQL document.
/// - `FieldsInSetCanMerge(set)` must be true.
///
/// `FieldsInSetCanMerge(set)`:
/// 1. Let `fieldsForName` be the set of selections with a given response name
///    in `set` including visiting fragments and inline fragments.
/// 2. Given each pair of members `fieldA` and `fieldB` in `fieldsForName`:
///    a. `SameResponseShape(fieldA, fieldB)` must be true.
///    b. If the parent types of `fieldA` and `fieldB` are equal or if either is
///       not an Object Type:
///       i.   `fieldA` and `fieldB` must have identical field names.
///       ii.  `fieldA` and `fieldB` must have identical sets of arguments.
///       iii. Let `mergedSet` be the result of adding the selection set of
///            `fieldA` and the selection set of `fieldB`.
///       iv. `FieldsInSetCanMerge(mergedSet)` must be `true`.
///
/// `SameResponseShape(fieldA, fieldB)`:
/// 1.  Let `typeA` be the return type of `fieldA`.
/// 2.  Let `typeB` be the return type of `fieldB`.
/// 3.  If `typeA` or `typeB` is Non-Null.
///     a. If `typeA` or `typeB` is nullable, return `false`.
///     b. Let `typeA` be the nullable type of `typeA`.
///     c. Let `typeB` be the nullable type of `typeB`.
/// 4.  If `typeA` or `typeB` is List.
///     a. If `typeA` or `typeB` is not List, return `false`.
///     b. Let `typeA` be the item type of `typeA`
///     c. Let `typeB` be the item type of `typeB`
///     d. Repeat from step 3.
/// 5.  If `typeA` or `typeB` is Scalar or Enum.
///     a. If `typeA` and `typeB` are the same type return true, otherwise return
///        false.
/// 6.  Assert: `typeA` and `typeB` are both composite types.
/// 7.  Let `mergedSet` be the result of adding the selection set of `fieldA` and
///     the selection set of `fieldB`.
/// 8.  Let `fieldsForName` be the set of selections with a given response name
///     in `mergedSet` including visiting fragments and inline fragments.
/// 9.  Given each pair of members `subfieldA` and `subfieldB` in
///     `fieldsForName`:
///     a. If `SameResponseShape(subfieldA, subfieldB)` is `false`, return
///        `false`.
/// 10. Return `true`.
pub struct FieldSelectionMerging;

impl FieldSelectionMerging {
    fn check_fields_in_set_can_merge<'v, 'a, T>(
        &self,
        document: &'v query::Document<'a, T>,
        schema: &'v schema::Document<'a, T>,
        set: HashMap<&'v str, Vec<(&'v Field<'a, T>, &'v str)>>,
        scope: &Scope<'_, 'v>,
        errors: &mut Vec<Error<'v, 'a, T>>,
    ) -> bool
    where
        'a: 'v,
        T: Text<'a>,
    {
        let mut valid = true;

        for (key, fields) in set {
            let (field_a, parent_a) = &fields[0];

            for (field_b, parent_b) in fields.iter().skip(1) {
                let (parent_a, parent_b) = match schema
                    .type_definition(parent_a)
                    .zip(schema.type_definition(parent_b))
                {
                    Some(types) => types,
                    None => continue,
                };

                let (type_a, type_b) = match parent_a
                    .field(&field_a.name)
                    .zip(parent_b.field(&field_b.name))
                {
                    Some((field_a, field_b)) => (&field_a.field_type, &field_b.field_type),
                    None => continue,
                };

                if !self.check_same_response_shape(
                    document, schema, key, field_a, type_a, field_b, type_b, scope, errors,
                ) {
                    valid = false;
                    break;
                }

                if parent_a.name().as_ref() != parent_b.name().as_ref()
                    && parent_a.is_object_type()
                    && parent_b.is_object_type()
                {
                    continue;
                }

                if field_a.name.as_ref() != field_b.name.as_ref() {
                    errors.push(Error::UnmergedFieldName {
                        name: key,
                        first_name: field_a.name.as_ref(),
                        first_span: field_a.span(),
                        second_name: field_b.name.as_ref(),
                        second_span: field_b.span(),
                    });
                    valid = false;
                    break;
                }

                if !field_a.is_equal(&field_b) {
                    errors.push(Error::UnmergedFieldArguments {
                        name: key,
                        first_name: field_a.name.as_ref(),
                        first_span: field_a.span(),
                        second_name: field_b.name.as_ref(),
                        second_span: field_b.span(),
                    });
                    valid = false;
                    continue;
                }

                let mut merged_set = HashMap::new();

                FieldSelectionMergingCollector { document }.traverse_selection_set(
                    &field_a.selection_set,
                    schema,
                    scope,
                    &mut merged_set,
                );

                FieldSelectionMergingCollector { document }.traverse_selection_set(
                    &field_a.selection_set,
                    schema,
                    scope,
                    &mut merged_set,
                );

                if !self.check_fields_in_set_can_merge(document, schema, merged_set, scope, errors)
                {
                    valid = false;
                    break;
                }
            }
        }

        valid
    }

    fn check_same_response_shape<'v, 'a, T>(
        &self,
        document: &'v query::Document<'a, T>,
        schema: &'v schema::Document<'a, T>,
        key: &'v str,
        field_a: &'v Field<'a, T>,
        type_a: &'v Type<'a, T>,
        field_b: &'v Field<'a, T>,
        type_b: &'v Type<'a, T>,
        scope: &Scope<'_, 'v>,
        errors: &mut Vec<Error<'v, 'a, T>>,
    ) -> bool
    where
        'a: 'v,
        T: Text<'a>,
    {
        let (orig_type_a, orig_type_b) = (type_a, type_b);

        let (type_a, type_b) = match (type_a, type_b) {
            (Type::NonNullType(type_a), Type::NonNullType(type_b))
            | (Type::ListType(type_a), Type::ListType(type_b)) => {
                return self.check_same_response_shape(
                    document, schema, key, field_a, type_a, field_b, type_b, scope, errors,
                )
            }
            (Type::NonNullType(_) | Type::ListType(_), _)
            | (_, Type::NonNullType(_) | Type::ListType(_)) => {
                errors.push(Error::UnmergedFieldType {
                    name: key,
                    first_name: field_a.name.as_ref(),
                    first_span: field_a.span(),
                    first_type: type_a,
                    second_name: field_b.name.as_ref(),
                    second_span: field_b.span(),
                    second_type: type_b,
                });
                return false;
            }
            (Type::NamedType(type_a), Type::NamedType(type_b)) => (type_a, type_b),
        };

        let (type_a, type_b) = match schema
            .type_definition(type_a)
            .zip(schema.type_definition(type_b))
        {
            Some(types) => types,
            None => return true,
        };

        if !type_a.is_composite() || !type_b.is_composite() {
            if type_a.name().as_ref() != type_b.name().as_ref() {
                errors.push(Error::UnmergedFieldType {
                    name: key,
                    first_name: field_a.name.as_ref(),
                    first_span: field_a.span(),
                    first_type: orig_type_a,
                    second_name: field_b.name.as_ref(),
                    second_span: field_b.span(),
                    second_type: orig_type_b,
                });

                return false;
            }

            return true;
        }

        let mut merged_set = HashMap::new();

        FieldSelectionMergingCollector { document }.traverse_selection_set(
            &field_a.selection_set,
            schema,
            scope,
            &mut merged_set,
        );

        FieldSelectionMergingCollector { document }.traverse_selection_set(
            &field_a.selection_set,
            schema,
            scope,
            &mut merged_set,
        );

        let mut valid = true;

        for (key, fields) in merged_set {
            let (field_a, parent_a) = &fields[0];

            for (field_b, parent_b) in fields.iter().skip(1) {
                let (parent_a, parent_b) = match schema
                    .type_definition(parent_a)
                    .zip(schema.type_definition(parent_b))
                {
                    Some(types) => types,
                    None => continue,
                };

                let (type_a, type_b) = match parent_a
                    .field(&field_a.name)
                    .zip(parent_b.field(&field_b.name))
                {
                    Some((field_a, field_b)) => (&field_a.field_type, &field_b.field_type),
                    None => continue,
                };

                if !self.check_same_response_shape(
                    document, schema, key, field_a, type_a, field_b, type_b, scope, errors,
                ) {
                    valid = false;
                    break;
                }
            }
        }

        valid
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for FieldSelectionMerging
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
            let mut set = HashMap::new();
            FieldSelectionMergingCollector { document }
                .traverse_definition(definition, schema, scope, &mut set);

            self.check_fields_in_set_can_merge(document, schema, set, scope, accumulator);
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FieldSelectionMerging
where
    'a: 'v,
    T: Text<'a>,
{
}

struct FieldSelectionMergingCollector<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    document: &'v query::Document<'a, T>,
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for FieldSelectionMergingCollector<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = HashMap<&'v str, Vec<(&'v Field<'a, T>, &'v str)>>;

    fn visit_field(
        &self,
        field: &'v query::Field<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let response_key = field.alias.as_ref().unwrap_or(&field.name).as_ref();
        accumulator
            .entry(response_key)
            .or_default()
            .push((field, scope.ty()));
    }

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v query::FragmentSpread<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match self
            .document
            .fragment_definition(fragment_spread.fragment_name.as_ref())
        {
            Some(fragment_definition) => {
                self.traverse_fragment_definition(fragment_definition, schema, scope, accumulator)
            }
            None => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for FieldSelectionMergingCollector<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    fn traverse_field(
        &self,
        field: &'v query::Field<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_field(field, schema, scope, accumulator);
    }
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    enum CatCommand {
        JUMP
    }

    type Cat {
        meowVolume: Int!

        doesKnowCommand(catCommand: CatCommand): Boolean!
    }

    enum DogCommand {
        SIT
        HEEL
    }

    type Dog {
        name: String!
        nickname: String!
        barkVolume: Int!

        doesKnowCommand(dogCommand: DogCommand): Boolean!
    }

    union Pet = Cat | Dog

    type Query {
        dog: Dog!
    }
    "#;

    #[test]
    fn test_field_selection_merging_118() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment mergeIdenticalFields on Dog {
            name
            name
        }

        fragment mergeIdenticalAliasesAndFields on Dog {
            otherName: name
            otherName: name
        }

        query {
            dog {
                ...mergeIdenticalFields
                ...mergeIdenticalAliasesAndFields
            }
        }
        "#,
        );
    }

    #[test]
    fn test_field_selection_merging_119() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment conflictingBecauseAlias on Dog {
            name: nickname
            name
        }

        {
            dog {
                ... conflictingBecauseAlias
            }
        }
        "#,
            r#"
        Error: 5.3.2 Field Selection Merging

          × Cannot merge two different fields into same response key `name`.
            ╭────
          1 │ fragment conflictingBecauseAlias on Dog {
          2 │     name: nickname
            ·     ─┬──
            ·      ╰── First, field `nickname` is selected as `name` here ...
            ·
          3 │     name
            ·     ─┬──
            ·      ╰── ... and then field `name` is also selected as `name` here.
            ·
          4 │ }
          5 │ 
          6 │ {
          7 │     dog {
          8 │         ... conflictingBecauseAlias
          9 │     }
         10 │ }
            ╰────
        "#,
        )
    }

    #[test]
    fn test_field_selection_merging_120() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment mergeIdenticalFieldsWithIdenticalArgs on Dog {
            doesKnowCommand(dogCommand: SIT)
            doesKnowCommand(dogCommand: SIT)
        }

        query {
            dog {
                ...mergeIdenticalFieldsWithIdenticalArgs
            }
        }
        "#,
        );

        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment mergeIdenticalFieldsWithIdenticalValues on Dog {
            doesKnowCommand(dogCommand: $dogCommand)
            doesKnowCommand(dogCommand: $dogCommand)
        }

        query bar($dogCommand: DogCommand!) {
            dog {
                ...mergeIdenticalFieldsWithIdenticalValues
            }
        }
        "#,
        )
    }

    #[test]
    fn test_field_selection_merging_121() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment conflictingArgsOnValues on Dog {
            doesKnowCommand(dogCommand: SIT)
            doesKnowCommand(dogCommand: HEEL)
        }

        query {
            dog {
                ...conflictingArgsOnValues
            }
        }
        "#,
            r#"
        Error: 5.3.2 Field Selection Merging

          × Cannot merge two fields with different arguments into same response key `doesKnowCommand`.
            ╭────
          1 │ fragment conflictingArgsOnValues on Dog {
          2 │     doesKnowCommand(dogCommand: SIT)
            ·     ───────┬───────
            ·            ╰── First, field `doesKnowCommand` is selected as `doesKnowCommand` here ...
            ·
          3 │     doesKnowCommand(dogCommand: HEEL)
            ·     ───────┬───────
            ·            ╰── ... and then the same field with different arguments is also selected as `doesKnowCommand` here.
            ·
          4 │ }
          5 │ 
          6 │ query {
          7 │     dog {
          8 │         ...conflictingArgsOnValues
          9 │     }
         10 │ }
            ╰────
        "#,
        );

        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment conflictingArgsValueAndVar on Dog {
            doesKnowCommand(dogCommand: SIT)
            doesKnowCommand(dogCommand: $dogCommand)
        }

        query ($dogCommand: DogCommand!) {
            dog {
                ...conflictingArgsValueAndVar
            }
        }
        "#,
            r#"
        Error: 5.3.2 Field Selection Merging

          × Cannot merge two fields with different arguments into same response key `doesKnowCommand`.
            ╭────
          1 │ fragment conflictingArgsValueAndVar on Dog {
          2 │     doesKnowCommand(dogCommand: SIT)
            ·     ───────┬───────
            ·            ╰── First, field `doesKnowCommand` is selected as `doesKnowCommand` here ...
            ·
          3 │     doesKnowCommand(dogCommand: $dogCommand)
            ·     ───────┬───────
            ·            ╰── ... and then the same field with different arguments is also selected as `doesKnowCommand` here.
            ·
          4 │ }
          5 │ 
          6 │ query ($dogCommand: DogCommand!) {
          7 │     dog {
          8 │         ...conflictingArgsValueAndVar
          9 │     }
         10 │ }
            ╰────
        "#,
        );

        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment conflictingArgsWithVars on Dog {
            doesKnowCommand(dogCommand: $varOne)
            doesKnowCommand(dogCommand: $varTwo)
        }

        query ($varOne: DogCommand!, $varTwo: DogCommand!) {
            dog {
                ...conflictingArgsWithVars
            }
        }
        "#,
            r#"
        Error: 5.3.2 Field Selection Merging
        
          × Cannot merge two fields with different arguments into same response key `doesKnowCommand`.
            ╭────
          1 │ fragment conflictingArgsWithVars on Dog {
          2 │     doesKnowCommand(dogCommand: $varOne)
            ·     ───────┬───────
            ·            ╰── First, field `doesKnowCommand` is selected as `doesKnowCommand` here ...
            ·
          3 │     doesKnowCommand(dogCommand: $varTwo)
            ·     ───────┬───────
            ·            ╰── ... and then the same field with different arguments is also selected as `doesKnowCommand` here.
            ·
          4 │ }
          5 │ 
          6 │ query ($varOne: DogCommand!, $varTwo: DogCommand!) {
          7 │     dog {
          8 │         ...conflictingArgsWithVars
          9 │     }
         10 │ }
            ╰────
        "#,
        );

        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment differingArgs on Dog {
            doesKnowCommand(dogCommand: SIT)
            doesKnowCommand
        }

        query {
            dog {
                ...differingArgs
            }
        }
        "#,
            r#"
        Error: 5.3.2 Field Selection Merging

          × Cannot merge two fields with different arguments into same response key `doesKnowCommand`.
            ╭────
          1 │ fragment differingArgs on Dog {
          2 │     doesKnowCommand(dogCommand: SIT)
            ·     ───────┬───────
            ·            ╰── First, field `doesKnowCommand` is selected as `doesKnowCommand` here ...
            ·
          3 │     doesKnowCommand
            ·     ───────┬───────
            ·            ╰── ... and then the same field with different arguments is also selected as `doesKnowCommand` here.
            ·
          4 │ }
          5 │ 
          6 │ query {
          7 │     dog {
          8 │         ...differingArgs
          9 │     }
         10 │ }
            ╰────
        "#,
        );
    }

    #[test]
    fn test_field_selection_merging_122() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment safeDifferingFields on Pet {
            ... on Dog {
                volume: barkVolume
            }
            ... on Cat {
                volume: meowVolume
            }
        }

        query {
            dog {
                ...safeDifferingFields
            }
        }
        "#,
        );

        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment safeDifferingArgs on Pet {
            ... on Dog {
                doesKnowCommand(dogCommand: SIT)
            }
            ... on Cat {
                doesKnowCommand(catCommand: JUMP)
            }
        }

        query {
            dog {
                ...safeDifferingArgs
            }
        }
        "#,
        );
    }

    #[test]
    fn test_field_selection_merging_123() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment conflictingDifferingResponses on Pet {
            ... on Dog {
                someValue: nickname
            }

            ... on Cat {
                someValue: meowVolume
            }
        }

        query {
            dog {
                ...conflictingDifferingResponses
            }
        }
        "#,
            r#"
        Error: 5.3.2 Field Selection Merging
        
          × Cannot merge two fields with different types into same response key `someValue`.
            ╭────
          1 │ fragment conflictingDifferingResponses on Pet {
          2 │     ... on Dog {
          3 │         someValue: nickname
            ·         ────┬────
            ·             ╰── First, field `nickname` of type `String` is selected as `someValue` here ...
            ·
          4 │     }
          5 │ 
          6 │     ... on Cat {
          7 │         someValue: meowVolume
            ·         ────┬────
            ·             ╰── ... and then field `meowVolume` of type `Int` is also selected as `someValue` here.
            ·
          8 │     }
          9 │ }
         10 │ 
         11 │ query {
         12 │     dog {
         13 │         ...conflictingDifferingResponses
         14 │     }
         15 │ }
            ╰────
        "#,
        )
    }
}

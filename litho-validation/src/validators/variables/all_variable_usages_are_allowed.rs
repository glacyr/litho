use std::collections::HashMap;

use graphql_parser::query::{Definition, Document, Text, Type, Value, VariableDefinition};
use graphql_parser::schema::{self};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.8.5 All Variable Usages Are Allowed
/// ## Formal Specification
/// - For each `operation` in `document`:
/// - Let `variableUsages` be all usages transitively included in the `operation`.
/// - For each `variableUsage` in `variableUsages`:
///   - Let `variableName` be the name of `variableUsage`.
///   - Let `variableDefinition` be the `VariableDefinition` named
///     `variableName` defined within `operation`.
///   - `IsVariableUsageAllowed(variableDefinition, variableUsage)` must be `true`.
///
/// `IsVariableUsageAllowed(variableDefinition, variableUsage)`:
/// 1. Let `variableType` be the expected type of `variableDefinition`.
/// 2. Let `locationType` be the expected type of the `Argument`, `ObjectField`,
///    or `ListValue` entry where `variableUsage` is located.
/// 3. If `locationType` is a non-null type AND `variableType` is NOT a non-null
///    type:
///    a. Let `hasNonNullVariableDefaultValue` be `true` if a default value
///       exists for `variableDefinition` and is not the value `null`.
///    b. Let `hasLocationDefaultValue` be `true` if a default value exists for
///       the `Argument` or `ObjectField` where `variableUsage` is located.
///    c. If `hasNonNullVariableDefaultValue` is NOT `true` AND
///       `hasLocationDefaultValue` is NOT `true`, return `false`.
///    d. Let `nullableLocationType` be the unwrapped type of `locationType`.
///    e. Return `AreTypesCompatible(variableType, nullableLocationType)`.
///  4. Return `AreTypesCompatible(variableType, locationType)`.
///
/// `AreTypesCompatible(variableType, locationType)`:
/// 1. If `locationType` is a non-null type:
///    a. If `variableType` is NOT a non-null type, return `false`.
///    b. Let `nullableLocationType` be the unwrapped nullable type of
///       `locationType`.
///    c. Let `nullableVariableType` be the unwrapped nullable type of
///       `variableType`.
///    d. Return `AreTypesCompatible(nullableVariableType, nullableLocationType)`.
/// 2. Otherwise, if `variableType` is a non-null type:
///    a. Let `nullableVariableType` be the nullable type of `variableType`.
///    b. Return `AreTypesCompatible(nullableVariableType, locationType)`.
/// 3. Otherwise, if `locationType` is a list type:
///    a. If `variableType` is NOT a list type, return `false`.
///    b. Let `itemLocationType` be the unwrapped item type of `locationType`.
///    c. Let `itemVariableType` be the unwrapped item type of `variableType`.
///    d. Return `AreTypesCompatible(itemVariableType, itemLocationType)`.
/// 4. Otherwise, if `variableType` is a list type, return `false`.
/// 5. Return `true` if `variableType` and `locationType` are identical,
///    otherwise `false`.
///
/// ## Explanatory Text
/// Variable usages must be compatible with the arguments they are passed to.
///
/// Validation failures occur when variables are used in the context of types
/// that are complete mismatches, or if a nullable type in a variable is passed
/// to a non-null argument type.
///
/// Types must match:
/// ```graphql
/// query intCannotGoIntoBoolean($intArg: Int) {
///   arguments {
///     booleanArgField(booleanArg: $intArg)
///   }
/// }
/// ```
///
/// `$intArg` typed as `Int` cannot be used as an argument to `booleanArg`,
/// typed as `Boolean`.
///
/// List cardinality must also be the same. For example, lists cannot be passed
/// into singular values.
/// ```graphql
/// query booleanListCannotGoIntoBoolean($booleanListArg: [Boolean]) {
///   arguments {
///     booleanArgField(booleanArg: $booleanListArg)
///   }
/// }
/// ```
///
/// Nullability must also be respected. In general a nullable variable cannot be
/// passed to a non-null argument.
/// ```graphql
/// query booleanArgQuery($booleanArg: Boolean) {
///   arguments {
///     nonNullBooleanArgField(nonNullBooleanArg: $booleanArg)
///   }
/// }
/// ```
///
/// For list types, the same rules around nullability apply to both outer types
/// and inner types. A nullable list cannot be passed to a non-null list, and a
/// list of nullable values cannot be passed to a list of non-null values. The
/// following is valid:
/// ```graphql
/// query nonNullListToList($nonNullBooleanList: [Boolean]!) {
///   arguments {
///     booleanListArgField(booleanListArg: $nonNullBooleanList)
///   }
/// }
/// ```
///
/// However, a nullable list cannot be passed to a non-null list:
/// ```graphql
/// query listToNonNullList($booleanList: [Boolean]) {
///   arguments {
///     nonNullBooleanListField(nonNullBooleanListArg: $booleanList)
///   }
/// }
/// ```
///
/// This would fail validation because `[T]` cannot be passed to a `[T]!`.
/// Similarly a `[T]` cannot be passed to a `[T!]`.
///
/// ### Allowing optional variables when default values exist
/// A notable exception to typical variable type compatibility is allowing a
/// variable definition with a nullable type to be provided to a non-null
/// location as either that variable or that location provides a default value.
///
/// In the example below, an optional variable `$booleanArg` is allowed to be
/// used in the non-null argument `optionalBooleanArg` because the field
/// argument is optional since it provides a default value in the schema.
/// ```graphql
/// query booleanArgQueryWithDefault($booleanArg: Boolean) {
///   arguments {
///     optionalNonNullBooleanArgField(optionalBooleanArg: $booleanArg)
///   }
/// }
/// ```
///
/// In the example below, an optional variable `$booleanArg` is allowed to be
/// used in the non-null argument (`nonNullBooleanArg`) because the variable
/// provides a default value in the operation. This behavior is explicitly
/// supported for compatibility with earlier editions of this specification.
/// GraphQL authoring tools may wish to report this as a warning with the
/// suggestion to replace `Boolean` with `Boolean!` to avoid ambiguity.
/// ```graphql
/// query booleanArgQueryWithDefault($booleanArg: Boolean = true) {
///   arguments {
///     nonNullBooleanArgField(nonNullBooleanArg: $booleanArg)
///   }
/// }
/// ```
///
/// Note: The value `null` could still be provided to such a variable at
/// runtime. A non-null argument must raise a field error if provided a `null`
/// value.
pub struct AllVariableUsagesAreAllowed;

impl<'v, 'a, T> Visitor<'v, 'a, T> for AllVariableUsagesAreAllowed
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_document(
        &self,
        document: &'v Document<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        for definition in document.definitions.iter() {
            if let Definition::Operation(definition) = definition {
                let variables = definition
                    .variable_definitions()
                    .iter()
                    .map(|var| (var.name.as_ref(), var))
                    .collect();

                AllVariableUsagesAreAllowedInner {
                    document,
                    variables,
                }
                .traverse_selection_set(
                    definition.selection_set(),
                    schema,
                    scope,
                    accumulator,
                )
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for AllVariableUsagesAreAllowed
where
    'a: 'v,
    T: Text<'a>,
{
}

struct AllVariableUsagesAreAllowedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    document: &'v Document<'a, T>,
    variables: HashMap<&'v str, &'v VariableDefinition<'a, T>>,
}

impl<'v, 'a, T> AllVariableUsagesAreAllowedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    fn variable_usage_allowed(
        &self,
        variable_definition: &'v VariableDefinition<'a, T>,
        location_type: &'v Type<'a, T>,
        location_default_value: bool,
    ) -> bool {
        match (&variable_definition.var_type, location_type) {
            (Type::ListType(_) | Type::NamedType(_), Type::NonNullType(_)) => {
                match (
                    !variable_definition.default_value.is_null(),
                    location_default_value,
                ) {
                    (false, false) => false,
                    (_, _) => {
                        self.types_compatible(&variable_definition.var_type, location_type.unwrap())
                    }
                }
            }
            (variable_type, location_type) => self.types_compatible(variable_type, location_type),
        }
    }

    fn types_compatible(
        &self,
        variable_type: &'v Type<'a, T>,
        location_type: &'v Type<'a, T>,
    ) -> bool {
        match (variable_type, location_type) {
            (Type::NonNullType(variable_type), Type::NonNullType(location_type))
            | (Type::ListType(variable_type), Type::ListType(location_type)) => {
                self.types_compatible(variable_type, location_type)
            }
            (Type::NonNullType(variable_type), location_type) => {
                self.types_compatible(variable_type, location_type)
            }
            (Type::NamedType(variable_type), Type::NamedType(location_type)) => {
                variable_type.as_ref() == location_type.as_ref()
            }
            _ => false,
        }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for AllVariableUsagesAreAllowedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_field(
        &self,
        field: &'v graphql_parser::query::Field<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let definition = match schema
            .type_definition(scope.ty())
            .and_then(|ty| ty.field(&field.name))
        {
            Some(field) => field,
            None => return,
        };

        for (key, value) in field.arguments.iter() {
            let variable = match value {
                Value::Variable(variable) => variable.as_ref(),
                _ => continue,
            };

            let variable = match self.variables.get(variable) {
                Some(variable) => variable,
                None => continue,
            };

            let input_value = match definition.argument(key.as_ref()) {
                Some(value) => value,
                None => continue,
            };

            if !self.variable_usage_allowed(
                &variable,
                &input_value.value_type,
                !input_value.default_value.is_null(),
            ) {
                accumulator.push(Error::IncompatibleVariableType {
                    name: key.as_ref(),
                    span: field.span(),
                    variable_name: variable.name.as_ref(),
                    variable_span: variable.span(),
                    expected_ty: &input_value.value_type,
                    actual_ty: &variable.var_type,
                })
            }
        }
    }

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v graphql_parser::query::FragmentSpread<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match self
            .document
            .fragment_definition(fragment_spread.fragment_name.as_ref())
        {
            Some(definition) => {
                self.traverse_fragment_definition(definition, schema, scope, accumulator)
            }
            None => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for AllVariableUsagesAreAllowedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    type Arguments {
        booleanArgField(booleanArg: Boolean): Boolean
        nonNullBooleanArgField(nonNullBooleanArg: Boolean!): Boolean!
        booleanListArgField(booleanListArg: [Boolean]): [Boolean]
        nonNullBooleanListField(nonNullBooleanListArg: [Boolean]!): [Boolean]!
        optionalNonNullBooleanArgField(optionalBooleanArg: Boolean! = false): Boolean!
    }

    type Query {
        arguments: Arguments!
    }
    "#;

    #[test]
    fn test_all_variable_usages_are_allowed_181() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query intCannotGoIntoBoolean($intArg: Int) {
            arguments {
                booleanArgField(booleanArg: $intArg)
            }
        }
        "#,
            r#"
        Error: 5.8.5 All Variable Usages Are Allowed

          × Variable `intArg` has incompatible type.
           ╭────
         1 │ query intCannotGoIntoBoolean($intArg: Int) {
           ·                              ───┬───
           ·                                 ╰── Variable `intArg` is defined here as type `Int` ...
           ·
         2 │     arguments {
         3 │         booleanArgField(booleanArg: $intArg)
           ·         ───────┬───────
           ·                ╰── ... but is used here as argument `booleanArg` of type `Boolean`.
           ·
         4 │     }
         5 │ }
           ╰────
        "#,
        );
    }

    #[test]
    fn test_all_variable_usages_are_allowed_182() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query booleanListCannotGoIntoBoolean($booleanListArg: [Boolean]) {
            arguments {
                booleanArgField(booleanArg: $booleanListArg)
            }
        }
        "#,
            r#"
        Error: 5.8.5 All Variable Usages Are Allowed

          × Variable `booleanListArg` has incompatible type.
           ╭────
         1 │ query booleanListCannotGoIntoBoolean($booleanListArg: [Boolean]) {
           ·                                      ───────┬───────
           ·                                             ╰── Variable `booleanListArg` is defined here as type `[Boolean]` ...
           ·
         2 │     arguments {
         3 │         booleanArgField(booleanArg: $booleanListArg)
           ·         ───────┬───────
           ·                ╰── ... but is used here as argument `booleanArg` of type `Boolean`.
           ·
         4 │     }
         5 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_all_variable_usages_are_allowed_183() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query booleanArgQuery($booleanArg: Boolean) {
            arguments {
                nonNullBooleanArgField(nonNullBooleanArg: $booleanArg)
            }
        }
        "#,
            r#"
        Error: 5.8.5 All Variable Usages Are Allowed

          × Variable `booleanArg` has incompatible type.
           ╭────
         1 │ query booleanArgQuery($booleanArg: Boolean) {
           ·                       ─────┬─────
           ·                            ╰── Variable `booleanArg` is defined here as type `Boolean` ...
           ·
         2 │     arguments {
         3 │         nonNullBooleanArgField(nonNullBooleanArg: $booleanArg)
           ·         ──────────┬───────────
           ·                   ╰── ... but is used here as argument `nonNullBooleanArg` of type `Boolean!`.
           ·
         4 │     }
         5 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_all_variable_usages_are_allowed_184() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query nonNullListToList($nonNullBooleanList: [Boolean]!) {
            arguments {
                booleanListArgField(booleanListArg: $nonNullBooleanList)
            }
        }
        "#,
        )
    }

    #[test]
    fn test_all_variable_usages_are_allowed_185() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query listToNonNullList($booleanList: [Boolean]) {
            arguments {
                nonNullBooleanListField(nonNullBooleanListArg: $booleanList)
            }
        }
        "#,
            r#"
        Error: 5.8.5 All Variable Usages Are Allowed
        
          × Variable `booleanList` has incompatible type.
           ╭────
         1 │ query listToNonNullList($booleanList: [Boolean]) {
           ·                         ─────┬──────
           ·                              ╰── Variable `booleanList` is defined here as type `[Boolean]` ...
           ·
         2 │     arguments {
         3 │         nonNullBooleanListField(nonNullBooleanListArg: $booleanList)
           ·         ───────────┬───────────
           ·                    ╰── ... but is used here as argument `nonNullBooleanListArg` of type `[Boolean]!`.
           ·
         4 │     }
         5 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_all_variable_usages_are_allowed_186() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query booleanArgQueryWithDefault($booleanArg: Boolean) {
            arguments {
                optionalNonNullBooleanArgField(optionalBooleanArg: $booleanArg)
            }
        }
        "#,
        )
    }

    #[test]
    fn test_all_variable_usages_are_allowed_187() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query booleanArgQueryWithDefault($booleanArg: Boolean = true) {
            arguments {
                nonNullBooleanArgField(nonNullBooleanArg: $booleanArg)
            }
        }
        "#,
        )
    }
}

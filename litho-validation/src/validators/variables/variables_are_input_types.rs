use graphql_parser::query::{Text, VariableDefinition};
use graphql_parser::schema;

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.8.2 Variables Are Input Types
/// ## Formal Specification
/// - For every `operation` in a `document`
/// - For every `variable` on each `operation`
///   - Let `variableType` be the type of `variable
///   - `IsInputType(variableType)` must be `true`
///
/// ## Explanatory Text
/// Variables can only be input types. Objects, unions, and interfaces cannot be
/// used as inputs.
///
/// For these examples, consider the following type system additions:
/// ```graphql
/// input ComplexInput {
///   name: String
///   owner: String
/// }
///
/// extend type Query {
///   findDog(complex: ComplexInput): Dog
///   booleanList(booleanListArg: [Boolean!]): Boolean
/// }
/// ```
///
/// The following operations are valid:
/// ```graphql
/// query takesBoolean($atOtherHomes: Boolean) {
///   dog {
///     isHouseTrained(atOtherHomes: $atOtherHomes)
///   }
/// }
///
/// query takesComplexInput($complexInput: ComplexInput) {
///   findDog(complex: $complexInput) {
///     name
///   }
/// }
///
/// query TakesListOfBooleanBang(booleans: [Boolean!]) {
///   booleanList(booleanListArg: $booleans)
/// }
/// ```
///
/// The following operations are invalid:
/// ```graphql
/// query takesCat($cat: Cat) {
///   # ...
/// }
///
/// query takesDogBang($dog: Dog!) {
///   # ...
/// }
///
/// query takesListOfPet($pets: [Pet]) {
///   # ...
/// }
///
/// query takesCatOrDog($catOrDog: CatOrDog) {
///   # ...
/// }
/// ```
pub struct VariablesAreInputTypes;

impl<'v, 'a, T> Visitor<'v, 'a, T> for VariablesAreInputTypes
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_variable_definition(
        &self,
        variable_definition: &'v VariableDefinition<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match schema.type_definition(&variable_definition.var_type.name()) {
            Some(ty) if !ty.is_input_type() => accumulator.push(Error::NonInputVariable {
                name: variable_definition.name.as_ref(),
                ty: ty.name().as_ref(),
                span: variable_definition.span(),
            }),
            _ => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for VariablesAreInputTypes
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    input ComplexInput {
        name: String
        owner: String
    }

    interface Pet {
        name: String!
    }

    type Cat implements Pet {
        name: String!
    }

    type Dog implements Pet {
        name: String!
        isHouseTrained(atOtherHomes: Boolean): Boolean!
    }

    union CatOrDog = Cat | Dog

    type Query {
        dog: Dog
        findDog(complex: ComplexInput): Dog
        booleanList(booleanListArg: [Boolean!]): Boolean
        outputCat(cat: Cat): Cat
        outputDogBang(dog: Dog!): Dog!
        outputListOfPets(pets: [Pet]): [Pet]
        outputCatOrDog(catOrDog: CatOrDog): CatOrDog
    }
    "#;

    #[test]
    fn test_variables_are_input_types_167_168() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query takesBoolean($atOtherHomes: Boolean) {
            dog {
                isHouseTrained(atOtherHomes: $atOtherHomes)
            }
        }

        query takesComplexInput($complexInput: ComplexInput) {
            findDog(complex: $complexInput) {
                name
            }
        }

        query TakesListOfBooleanBang($booleans: [Boolean!]) {
            booleanList(booleanListArg: $booleans)
        }
        "#,
        )
    }

    #[test]
    fn test_variables_are_input_types_169() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query takesCat($cat: Cat) {
            outputCat(cat: $cat) {
                name
            }
        }
        "#,
            r#"
        Error: 5.8.2 Variables Are Input Types

          × Variable `cat` isn't input type.
           ╭────
         1 │ query takesCat($cat: Cat) {
           ·                ─┬─
           ·                 ╰── Variable `cat` is defined here but `Cat` is not an input type.
           ·
         2 │     outputCat(cat: $cat) {
         3 │         name
         4 │     }
         5 │ }
           ╰────
        "#,
        );

        crate::tests::assert_err(
            SCHEMA,
            r#"
        query takesDogBang($dog: Dog!) {
            outputDogBang(dog: $dog) {
                name
            }
        }
        "#,
            r#"
        Error: 5.8.2 Variables Are Input Types

          × Variable `dog` isn't input type.
           ╭────
         1 │ query takesDogBang($dog: Dog!) {
           ·                    ─┬─
           ·                     ╰── Variable `dog` is defined here but `Dog` is not an input type.
           ·
         2 │     outputDogBang(dog: $dog) {
         3 │         name
         4 │     }
         5 │ }
           ╰────
        "#,
        );

        crate::tests::assert_err(
            SCHEMA,
            r#"
        query takesListOfPet($pets: [Pet]) {
            outputListOfPets(pets: $pets) {
                name
            }
        }
        "#,
            r#"
        Error: 5.8.2 Variables Are Input Types

          × Variable `pets` isn't input type.
           ╭────
         1 │ query takesListOfPet($pets: [Pet]) {
           ·                      ─┬──
           ·                       ╰── Variable `pets` is defined here but `Pet` is not an input type.
           ·
         2 │     outputListOfPets(pets: $pets) {
         3 │         name
         4 │     }
         5 │ }
           ╰────
        "#,
        );

        crate::tests::assert_err(
            SCHEMA,
            r#"
        query takesCatOrDog($catOrDog: CatOrDog) {
            outputCatOrDog(catOrDog: $catOrDog) {
                ... on Cat {
                    name
                }
            }
        }
        "#,
            r#"
        Error: 5.8.2 Variables Are Input Types
        
          × Variable `catOrDog` isn't input type.
           ╭────
         1 │ query takesCatOrDog($catOrDog: CatOrDog) {
           ·                     ───┬────
           ·                        ╰── Variable `catOrDog` is defined here but `CatOrDog` is not an input type.
           ·
         2 │     outputCatOrDog(catOrDog: $catOrDog) {
         3 │         ... on Cat {
         4 │             name
         5 │         }
         6 │     }
         7 │ }
           ╰────
        "#,
        );
    }
}

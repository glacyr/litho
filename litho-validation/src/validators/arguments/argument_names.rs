use graphql_parser::query::{Selection, Text};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.4.1 Argument Names
/// ## Formal Specification
/// - For each `argument` in the document
/// - Let `argumentName` be the Name of `argument`.
/// - Let `argumentDefinition` be the argument definition provided by the parent
///   field or definition named `argumentName`.
/// - `argumentDefinition` must exist.
///
/// ## Explanatory Text
/// Every argument provided to a field or directive must be defined in the set
/// of possible arguments of that field or directive.
///
/// For example the following are valid:
/// ```graphql
/// fragment argOnRequiredArg on Dog {
///   doesKnowCommand(dogCommand: SIT)
/// }
///
/// fragment argOnOptional on Dog {
///   isHousetrained(atOtherHomes: true) @include(if: true)
/// }
/// ```
///
/// the following is invalid since `command` is not defined on `DogCommand`.
/// ```graphql
/// fragment invalidArgName on Dog {
///   doesKnowCommand(command: CLEAN_UP_HOUSE)
/// }
/// ```
///
/// and this is also invalid as `unless` is not defined on `@include`.
/// ```graphql
/// fragment invalidArgName on Dog {
///   isHousetrained(atOtherHomes: true) @include(unless: false)
/// }
/// ```
///
/// In order to explore more complicated argument examples, let's add the
/// following to our type system:
/// ```graphql
/// type Arguments {
///   multipleReqs(x: Int!, y: Int!): Int!
///   booleanArgField(booleanArg: Boolean): Boolean
///   floatArgField(floatArg: Float): Float
///   intArgField(intArg: Int): Int
///   nonNullBooleanArgField(nonNullBooleanArg: Boolean!): Boolean!
///   booleanListArgField(booleanListArg: [Boolean]!): [Boolean]
///   optionalNonNullBooleanArgField(optionalBooleanArg: Boolean! = false): Boolean!
/// }
///
/// extend type Query {
///   arguments: Arguments
/// }
/// ```
///
/// Order does not matter in arguments. Therefore both the following example are
/// valid.
/// ```graphql
/// fragment multipleArgs on Arguments {
///   multipleReqs(x: 1, y: 2)
/// }
///
/// fragment multipleArgsReverseOrder on Arguments {
///   multipleReqs(y: 1, x: 2)
/// }
/// ```
pub struct ArgumentNames;

impl<'v, 'a, T> Visitor<'v, 'a, T> for ArgumentNames
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
                    let field = match ty.field(&selection.name) {
                        Some(field) => field,
                        None => continue,
                    };

                    for (name, _) in &selection.arguments {
                        if field
                            .arguments
                            .iter()
                            .find(|input| &input.name == name)
                            .is_none()
                        {
                            accumulator.push(Error::UndefinedArgument {
                                field_name: &selection.name,
                                field_span: selection.span(),
                                ty: ty.name().to_owned(),
                                name,
                            });
                        }
                    }
                }
                Selection::InlineFragment(_) | Selection::FragmentSpread(_) => continue,
            }
        }
    }

    fn visit_directive(
        &self,
        directive: &'v schema::Directive<'a, T>,
        schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let definition = match schema.directive_definition(&directive.name) {
            Some(definition) => definition,
            None => return,
        };

        for (name, _) in &directive.arguments {
            if definition
                .arguments
                .iter()
                .find(|input| &input.name == name)
                .is_none()
            {
                accumulator.push(Error::UndefinedDirectiveArgument {
                    directive_name: &directive.name,
                    directive_span: directive.span(),
                    name,
                });
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for ArgumentNames
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    directive @include(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT

    enum DogCommand {
        SIT
        CLEAN_UP_HOUSE
    }

    type Dog {
        doesKnowCommand(dogCommand: DogCommand): Boolean!
        isHouseTrained(atOtherHomes: Boolean!): Boolean!
    }

    type Query {
        dog: Dog
    }
    "#;

    #[test]
    fn test_argument_names_128() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        fragment argOnRequiredArg on Dog {
            doesKnowCommand(dogCommand: SIT)
        }

        {
            dog {
                ... argOnRequiredArg
            }
        }
        "#,
        );
    }

    #[test]
    fn test_argument_names_129() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment argOnRequiredArg on Dog {
            doesKnowCommand(command: CLEAN_UP_HOUSE)
        }

        {
            dog {
                ... argOnRequiredArg
            }
        }
        "#,
            r#"
        Error: 5.4.1 Argument Names

          × Given argument `command` is not defined for field `doesKnowCommand` of type `Dog`.
           ╭────
         1 │ fragment argOnRequiredArg on Dog {
         2 │     doesKnowCommand(command: CLEAN_UP_HOUSE)
           ·     ───────┬───────
           ·            ╰── Argument `command` does not exist on field `doesKnowCommand` of type `Dog`.
           ·
         3 │ }
         4 │ 
         5 │ {
         6 │     dog {
         7 │         ... argOnRequiredArg
         8 │     }
         9 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_argument_names_130() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        fragment argOnRequiredArg on Dog {
            isHouseTrained(atOtherHomes: true) @include(unless: false)
        }
        
        {
            dog {
                ... argOnRequiredArg
            }
        }
        "#,
            r#"
            Error: 5.4.1 Argument Names
            
              × Given argument `unless` is not defined for directive `include`.
               ╭────
             1 │ fragment argOnRequiredArg on Dog {
             2 │     isHouseTrained(atOtherHomes: true) @include(unless: false)
               ·                                        ───┬────
               ·                                           ╰── Argument `unless` does not exist on directive `include`.
               ·
             3 │ }
             4 │ 
             5 │ {
             6 │     dog {
             7 │         ... argOnRequiredArg
             8 │     }
             9 │ }
               ╰────
        "#,
        )
    }

    #[test]
    fn test_argument_names_131_132() {
        crate::tests::assert_ok(
            r#"
        type Arguments {
            multipleRequirements(x: Int!, y: Int!): Int!
            booleanArgField(booleanArg: Boolean): Boolean
            floatArgField(floatArg: Float): Float
            intArgField(intArg: Int): Int
            nonNullBooleanArgField(nonNullBooleanArg: Boolean!): Boolean!
            booleanListArgField(booleanListArg: [Boolean]!): [Boolean]
            optionalNonNullBooleanArgField(optionalBooleanArg: Boolean! = false): Boolean!
        }
          
        type Query {
            arguments: Arguments
        }
        "#,
            r#"
        fragment multipleArgs on Arguments {
            multipleRequirements(x: 1, y: 2)
        }
          
        fragment multipleArgsReverseOrder on Arguments {
            multipleRequirements(y: 2, x: 1)
        }

        {
            a: arguments {
                ...multipleArgs
            }

            b: arguments {
                ...multipleArgsReverseOrder
            }
        }
        "#,
        )
    }
}

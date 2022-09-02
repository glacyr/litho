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
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for ArgumentNames
where
    'a: 'v,
    T: Text<'a>,
{
}

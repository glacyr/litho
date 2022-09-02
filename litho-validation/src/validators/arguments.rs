use std::collections::HashSet;

use graphql_parser::query::{Selection, Text, Value};
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

/// # 5.4.2 Argument Uniqueness
///
/// Fields and directives treat arguments as a mapping of argument name to
/// value. More than one argument with the same name in an argument set is
/// ambiguous and invalid.
///
/// ## Formal Specification
/// - For each `argument` in the Document.
/// - Let `argumentName` be the Name of `argument`.
/// - Let `arguments` be all Arguments named `argumentName` in the Argument Set
///   which contains `argument`.
/// - `arguments` must be the set containing only `argument`.
pub struct ArgumentUniqueness;

impl<'v, 'a, T> Visitor<'v, 'a, T> for ArgumentUniqueness
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
                    let mut unique = HashSet::new();
                    let mut dups = HashSet::new();

                    for (name, _) in &selection.arguments {
                        if !unique.contains(&name.as_ref()) {
                            unique.insert(name.as_ref());
                        } else {
                            dups.insert(name.as_ref());
                        }
                    }

                    let mut dups = dups.into_iter().collect::<Vec<_>>();
                    dups.sort();

                    accumulator.extend(dups.into_iter().map(|name| Error::DuplicateArgumentName {
                        field_name: &selection.name,
                        field_span: selection.span(),
                        ty: ty.name().to_owned(),
                        name,
                    }));
                }
                Selection::InlineFragment(_) | Selection::FragmentSpread(_) => continue,
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for ArgumentUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
}

/// # 5.4.2.1 Required Arguments
/// ## Formal Specification
/// - For each Field or Directive in the document.
/// - Let `arguments` be the arguments provided by the Field or Directive.
/// - Let `argumentDefinitions` be the set of argument definitions of that Field
///   or Directive.
/// - For each `argumentDefinition` in `argumentDefinitions`:
///   - Let `type` be the expected type of `argumentDefinition`.
///   - Let `defaultValue` be the default value of `argumentDefinition`.
///   - If `type` is Non-Null and `defaultValue` does not exist:
///     - Let `argumentName` be the name of `argumentDefinition`.
///     - Let `argument` be the argument in `arguments` named `argumentName`
///     - `argument` must exist.
///     - Let `value` be the value of `argument`.
///     - `value` must not be the `null` literall.
///
/// ## Explanatory Text
/// Arguments can be required. An argument is required if the argument type is
/// non-null and does not have a default value. Otherwise, the argument is
/// optional.
///
/// For example the following are valid:
/// ```graphql
/// fragment goodBooleanArg on Arguments {
///   booleanArgField(booleanArg: true)
/// }
///
/// fragment goodNonNullArg on Arguments {
///   nonNullBooleanArgField(nonNullBooleanArg: true)
/// }
/// ```
///
/// The argument can be omitted from a field with a nullable argument.
///
/// Therefore the following fragment is valid:
/// ```graphql
/// fragment goodBooleanArgDefault on Arguments {
///   booleanArgField
/// }
/// ```
///
/// but this is not valid on a required argument.
/// ```graphql
/// fragment missingRequiredArg on Arguments {
///   nonNullBooleanArgField
/// }
/// ```
///
/// Providing the explicit value `null` is also not valid since required
/// arguments always have a non-null type.
/// ```graphql
/// fragment missingRequiredArg on Arguments {
///   nonNullBooleanArgField(nonNullBooleanArg: null)
/// }
/// ```
pub struct RequiredArguments;

impl<'v, 'a, T> Visitor<'v, 'a, T> for RequiredArguments
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
            let selection = match selection {
                Selection::Field(ref selection) => selection,
                Selection::FragmentSpread(_) | Selection::InlineFragment(_) => continue,
            };

            let field = match ty.field(&selection.name) {
                Some(field) => field,
                None => continue,
            };

            let arguments = selection
                .arguments
                .iter()
                .flat_map(|(name, value)| match value {
                    Value::Null => None,
                    _ => Some(name.as_ref()),
                })
                .collect::<HashSet<_>>();

            for arg in &field.arguments {
                match (
                    arg.value_type.is_required(),
                    arg.default_value.is_some(),
                    arguments.contains(arg.name.as_ref()),
                ) {
                    (false, _, _) | (_, true, _) | (true, _, true) => continue,
                    (true, false, false) => accumulator.push(Error::MissingRequiredArgument {
                        field_name: &selection.name,
                        field_span: selection.span(),
                        ty: ty.name().to_owned(),
                        name: &arg.name,
                    }),
                }
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for RequiredArguments
where
    'a: 'v,
    T: Text<'a>,
{
}

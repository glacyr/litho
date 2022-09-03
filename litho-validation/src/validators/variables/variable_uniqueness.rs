use std::collections::{HashMap, HashSet};

use graphql_parser::query::{Text, VariableDefinition};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.8.1 Variable Uniqueness
/// ## Formal Specification
/// - For every `operation` in the document
///   - For every `variable` defined on `operation`
///     - Let `variableName` be the name of `variable`
///     - Let `variables` be the set of all variables named `variableName` on
///       `operation`.
///     - `variables` must be a set of one
///
/// ## Explanatory Text
/// If any operation defines more than one variable with the same name, it is
/// ambiguous and invalid. It ia invalid even if the type of the duplicate
/// variable is the same.
/// ```graphql
/// query houseTrainedQuery($atOtherHomes: Boolean, $atOtherHomes: Boolean) {
///   dog {
///     isHouseTrained(atOtherHomes: $atOtherHomes)
///   }
/// }
/// ```
///
/// It is valid for multiple operations to define a variable with the same name.
/// If two operations reference the same fragment, it might actually be
/// necessary.
/// ```graphql
/// query A($atOtherHomes: Boolean) {
///   ...HouseTrainedFragment
/// }
///
/// query B($atOtherHomes: Boolean) {
///   ..HouseTrainedFragment
/// }
///
/// fragment HouseTrainedFragment on Query {
///   dog {
///     isHouseTrained(atOtherHomes: $atOtherHomes)
///   }
/// }
/// ```
pub struct VariableUniqueness;

impl<'v, 'a, T> Visitor<'v, 'a, T> for VariableUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_operation_definition(
        &self,
        operation_definition: &'v query::OperationDefinition<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut visited = HashMap::<&str, &VariableDefinition<'a, T>>::new();
        let mut errored = HashSet::new();

        for variable in operation_definition.variable_definitions().iter() {
            if let Some(existing) = visited.get(variable.name.as_ref()) {
                if errored.contains(variable.name.as_ref()) {
                    continue;
                }

                accumulator.push(Error::DuplicateVariableName {
                    name: variable.name.as_ref(),
                    first: existing.span(),
                    second: variable.span(),
                });
                errored.insert(variable.name.as_ref());
            } else {
                visited.insert(variable.name.as_ref(), variable);
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for VariableUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    type Dog {
        isHouseTrained(atOtherHomes: Boolean!): Boolean!
    }

    type Query {
        dog: Dog!
    }
    "#;

    #[test]
    fn test_variable_uniqueness_165() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query houseTrainedQuery($atOtherHomes: Boolean, $atOtherHomes: Boolean) {
            dog {
                isHouseTrained(atOtherHomes: $atOtherHomes)
            }
        }
        "#,
            r#"
        Error: 5.8.1 Variable Uniqueness

          × Variable `atOtherHomes` is defined twice.
           ╭────
         1 │ query houseTrainedQuery($atOtherHomes: Boolean, $atOtherHomes: Boolean) {
           ·                         ─────┬──────
           ·                              ╰── Variable `atOtherHomes` is first defined here ...
           ·
           ·                                                 ─────┬──────
           ·                                                      ╰── ... and later defined again here.
           ·
         2 │     dog {
         3 │         isHouseTrained(atOtherHomes: $atOtherHomes)
         4 │     }
         5 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_variable_uniqueness_166() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query A($atOtherHomes: Boolean) {
            ...HouseTrainedFragment
        }

        query B($atOtherHomes: Boolean) {
            ...HouseTrainedFragment
        }

        fragment HouseTrainedFragment on Query {
            dog {
                isHouseTrained(atOtherHomes: $atOtherHomes)
            }
        }
        "#,
        )
    }
}

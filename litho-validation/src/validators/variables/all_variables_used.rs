use std::collections::HashSet;

use graphql_parser::query::{Definition, Document, Text, Value};
use graphql_parser::schema;

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.8.4 All Variables Used
/// ## Formal Specification
/// - For every `operation` in the document.
/// - Let `variables` be the variables defined by that `operation`
/// - Each `variable` in `variables` must be used at least once in either the
///   operation scope itself or any fragment transitively referenced by that
///   operation.
///
/// ## Explanatory Text
/// All variables defined by an operation must be used in that operation or a
/// fragment transitively included by that operation. Unused variables cause a
/// validation error.
///
/// For example the following is invalid:
/// ```graphql
/// query variableUnused($atOtherHomes: Boolean) {
///   dog {
///     isHouseTrained
///   }
/// }
/// ```
/// because `$atOtherHomes` is not referenced.
///
/// These rules apply to transitive fragment spreads as well:
/// ```graphql
/// query variableUsedInFragment($atOtherHomes: Boolean) {
///   dog {
///     ...isHouseTrainedFragment
///   }
/// }
///
/// fragment isHouseTrainedFragment on Dog {
///   isHouseTrained(atOtherHomes: $atOtherHomes)
/// }
/// ```
///
/// The above is valid since `$atOtherHomes` is used in `isHouseTrainedFragment`
/// which is included by `variableUsedInFragment`.
///
/// If that fragment did not have a reference to `$atOtherHomes` it would be not
/// valid:
/// ```graphql
/// query variableNotUsedWithinFragment($atOtherHomes: Boolean) {
///   dog {
///     ...isHouseTrainedWithoutVariableFragment
///   }
/// }
///
/// fragment isHouseTrainedWithoutVariableFragment on Dog {
///   isHouseTrained
/// }
/// ```
///
/// All operations in a document must use all of their variables.
///
/// As a result, the following document does not validate.
/// ```graphql
/// query queryWithUsedVar($atOtherHomes: Boolean) {
///   dog {
///     ...isHouseTrainedFragment
///   }
/// }
///
/// fragment isHouseTrainedFragment on Dog {
///   isHouseTrained(atOtherHomes: $atOtherHomes)
/// }
/// ```
///
/// This document is not valid because `queryWithExtraVar` defines an extraneous
/// variable.
pub struct AllVariablesUsed;

impl<'v, 'a, T> Visitor<'v, 'a, T> for AllVariablesUsed
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
                let mut used = HashSet::new();

                AllVariablesUsedInner { document }.traverse_selection_set(
                    definition.selection_set(),
                    schema,
                    scope,
                    &mut used,
                );

                for variable in definition.variable_definitions().iter() {
                    if !used.contains(variable.name.as_ref()) {
                        accumulator.push(Error::UnusedVariable {
                            name: variable.name.as_ref(),
                            span: variable.span(),
                        })
                    }
                }
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for AllVariablesUsed
where
    'a: 'v,
    T: Text<'a>,
{
}

struct AllVariablesUsedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    document: &'v Document<'a, T>,
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for AllVariablesUsedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = HashSet<&'v str>;

    fn visit_value(
        &self,
        value: &'v Value<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match value {
            Value::Variable(variable) => {
                accumulator.insert(variable.as_ref());
            }
            _ => {}
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

impl<'v, 'a, T> Traverse<'v, 'a, T> for AllVariablesUsedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    type Dog {
        isHouseTrained(atOtherHomes: Boolean): Boolean!
    }

    type Query {
        dog: Dog
    }
    "#;

    #[test]
    fn test_all_variables_used_177() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query variableUnused($atOtherHomes: Boolean) {
            dog {
                isHouseTrained
            }
        }
        "#,
            r#"
        Error: 5.8.4 All Variables Used

          × Variable `atOtherHomes` is never used.
           ╭────
         1 │ query variableUnused($atOtherHomes: Boolean) {
           ·                      ──────┬──────
           ·                            ╰── Variable `atOtherHomes` is defined here but never used.
           ·
         2 │     dog {
         3 │         isHouseTrained
         4 │     }
         5 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_all_variables_used_178() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query variableUsedInFragment($atOtherHomes: Boolean) {
            dog {
                ...isHouseTrainedFragment
            }
        }

        fragment isHouseTrainedFragment on Dog {
            isHouseTrained(atOtherHomes: $atOtherHomes)
        }
        "#,
        )
    }

    #[test]
    fn test_all_variables_used_179() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query variableNotUsedWithinFragment($atOtherHomes: Boolean) {
            dog {
                ...isHouseTrainedWithoutVariableFragment
            }
        }

        fragment isHouseTrainedWithoutVariableFragment on Dog {
            isHouseTrained
        }
        "#,
            r#"
        Error: 5.8.4 All Variables Used

          × Variable `atOtherHomes` is never used.
           ╭────
         1 │ query variableNotUsedWithinFragment($atOtherHomes: Boolean) {
           ·                                     ──────┬──────
           ·                                           ╰── Variable `atOtherHomes` is defined here but never used.
           ·
         2 │     dog {
         3 │         ...isHouseTrainedWithoutVariableFragment
         4 │     }
         5 │ }
         6 │ 
         7 │ fragment isHouseTrainedWithoutVariableFragment on Dog {
         8 │     isHouseTrained
         9 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_180() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query queryWithUsedVar($atOtherHomes: Boolean) {
            dog {
                ...isHouseTrainedFragment
            }
        }

        query queryWithExtraVar($atOtherHomes: Boolean, $extra: Int) {
            dog {
                ...isHouseTrainedFragment
            }
        }

        fragment isHouseTrainedFragment on Dog {
            isHouseTrained(atOtherHomes: $atOtherHomes)
        }
        "#,
            r#"
        Error: 5.8.4 All Variables Used
        
          × Variable `extra` is never used.
            ╭────
          1 │ query queryWithUsedVar($atOtherHomes: Boolean) {
          2 │     dog {
          3 │         ...isHouseTrainedFragment
          4 │     }
          5 │ }
          6 │ 
          7 │ query queryWithExtraVar($atOtherHomes: Boolean, $extra: Int) {
            ·                                                 ──┬───
            ·                                                   ╰── Variable `extra` is defined here but never used.
            ·
          8 │     dog {
          9 │         ...isHouseTrainedFragment
         10 │     }
         11 │ }
         12 │ 
         13 │ fragment isHouseTrainedFragment on Dog {
         14 │     isHouseTrained(atOtherHomes: $atOtherHomes)
         15 │ }
            ╰────
        "#,
        )
    }
}

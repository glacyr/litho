use std::collections::HashSet;

use graphql_parser::query::{Definition, Document, Text, Value};
use graphql_parser::schema;

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.8.3 All Variable Uses Defined
/// ## Formal Specification
/// - For each `operation` in a document
///   - For each `variableUsage` in scope, variable must be in `operation`'s
///     variable list.
///   - Let `fragments` be every fragment referenced by that `operation`
///     transitively
///   - For each `fragment` in `fragments`
///     - For each `variableUsage` in scope of `fragment`, variable must be in
///       `operation`'s variable list.
///
/// ## Explanatory Text
/// Variables are scoped on a per-operation basis. That means that any variable
/// used within the context of an operation must be defined at the top level of
/// that operation
///
/// For example:
/// ```graphql
/// query variableIsDefined($atOtherHomes: Boolean) {
///   dog {
///     isHouseTrained(atOtherHomes: $atOtherHomes)
///   }
/// }
/// ```
///
/// is valid. `$atOtherHomes` is defined by the operation.
///
/// By contrast the following document is invalid:
/// ```graphql
/// query variableIsNotDefined {
///   dog {
///     isHouseTrained(atOtherHomes: $atOtherHomes)
///   }
/// }
/// ```
///
/// `$atOtherHomes` is not defined by the operation.
///
/// Fragments complicate this rule. Any fragment transitively included by an
/// operation has access to the variables defined by that operation. Fragments
/// can appear within multiple operations and therefore variable usages must
/// correspond to variable definitions in all of those operations.
///
/// For example the following is valid:
/// ```graphql
/// query variableIsDefinedUsedInSingleFragment($atOtherHomes: Boolean) {
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
/// since `isHouseTrainedFragment` is used within the context of the operation
/// `variableIsDefinedUsedInSingleFragment` and the variable is defined by that
/// operation.
///
/// On the other hand, if a fragment is included within an operation that does
/// not define a referenced variable, the document is invalid.
/// ```graphql
/// query variableIsNotDefinedUsedInSingleFragment {
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
/// This applies transitively as well, aso the following also fails:
/// ```graphql
/// query variableIsNotDefinedUsedInNestedFragment {
///   dog {
///     ...outerHouseTrainedFragment
///   }
/// }
///
/// fragment outerHouseTrainedFragment on Dog {
///   ...isHouseTrainedFragment
/// }
///
/// fragment isHouseTrainedFragment on Dog {
///   isHouseTrained(atOtherHomes: $atOtherHomes)
/// }
/// ```
///
/// Variables must be defined in all operations in which a fragment is used.
/// ```graphql
/// query houseTrainedQueryOne($atOtherHomes: Boolean) {
///   dog {
///     ...isHouseTrainedFragment
///   }
/// }
///
/// query houseTrainedQueryTwo($atOtherHomes: Boolean) {
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
/// However the following does not validate:
/// ```graphql
/// query houseTrainedQueryOne($atOtherHomes: Boolean) {
///   dog {
///     ...isHouseTrainedFragment
///   }
/// }
///
/// query houseTrainedQueryTwoNotDefined {
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
/// This is because `houseTrainedQueryTwoNotDefined` does not define a variable
/// `$atOtherHomes` but that variable is used by `isHouseTrainedFragment` which
/// is included in that operation.
pub struct AllVariableUsesDefined;

impl<'v, 'a, T> Visitor<'v, 'a, T> for AllVariableUsesDefined
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
                    .map(|var| var.name.as_ref())
                    .collect();

                AllVariableUsesDefinedInner {
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

impl<'v, 'a, T> Traverse<'v, 'a, T> for AllVariableUsesDefined
where
    'a: 'v,
    T: Text<'a>,
{
}

struct AllVariableUsesDefinedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    document: &'v Document<'a, T>,
    variables: HashSet<&'v str>,
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for AllVariableUsesDefinedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_value(
        &self,
        value: &'v Value<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match value {
            Value::Variable(variable) if !self.variables.contains(variable.as_ref()) => accumulator
                .push(Error::UndefinedVariable {
                    name: variable.as_ref(),
                    span: scope.span(),
                }),
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

impl<'v, 'a, T> Traverse<'v, 'a, T> for AllVariableUsesDefinedInner<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    type Dog {
        isHouseTrained(atOtherHomes: Boolean): Boolean
    }

    type Query {
        dog: Dog
    }
    "#;

    #[test]
    fn test_all_variable_uses_defined_170() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query variableIsDefined($atOtherHomes: Boolean) {
            dog {
                isHouseTrained(atOtherHomes: $atOtherHomes)
            }
        }
        "#,
        )
    }

    #[test]
    fn test_all_variable_uses_defined_171() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query variableIsNotDefined {
            dog {
                isHouseTrained(atOtherHomes: $atOtherHomes)
            }
        }
        "#,
            r#"
        Error: 5.8.3 All Variable Uses Defined

          × Variable `atOtherHomes` is not defined.
           ╭────
         1 │ query variableIsNotDefined {
         2 │     dog {
         3 │         isHouseTrained(atOtherHomes: $atOtherHomes)
           ·         ──────┬───────
           ·               ╰── Variable `atOtherHomes` is used here but not defined anywhere.
           ·
         4 │     }
         5 │ }
           ╰────
            "#,
        )
    }

    #[test]
    fn test_all_variable_uses_defined_172() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query variableIsDefinedUsedInSingleFragment($atOtherHomes: Boolean) {
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
    fn test_all_variable_uses_defined_173() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query variableIsNotDefinedUsedInSingleFragment {
            dog {
                ...isHouseTrainedFragment
            }
        }

        fragment isHouseTrainedFragment on Dog {
            isHouseTrained(atOtherHomes: $atOtherHomes)
        }
        "#,
            r#"
        Error: 5.8.3 All Variable Uses Defined

          × Variable `atOtherHomes` is not defined.
           ╭────
         1 │ query variableIsNotDefinedUsedInSingleFragment {
         2 │     dog {
         3 │         ...isHouseTrainedFragment
         4 │     }
         5 │ }
         6 │ 
         7 │ fragment isHouseTrainedFragment on Dog {
         8 │     isHouseTrained(atOtherHomes: $atOtherHomes)
           ·     ──────┬───────
           ·           ╰── Variable `atOtherHomes` is used here but not defined anywhere.
           ·
         9 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_all_variable_uses_defined_174() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query variableIsNotDefinedUsedInNestedFragment {
            dog {
                ...outerHouseTrainedFragment
            }
        }

        fragment outerHouseTrainedFragment on Dog {
            ...isHouseTrainedFragment
        }

        fragment isHouseTrainedFragment on Dog {
            isHouseTrained(atOtherHomes: $atOtherHomes)
        }
        "#,
            r#"
        Error: 5.8.3 All Variable Uses Defined

          × Variable `atOtherHomes` is not defined.
            ╭────
          1 │ query variableIsNotDefinedUsedInNestedFragment {
          2 │     dog {
          3 │         ...outerHouseTrainedFragment
          4 │     }
          5 │ }
          6 │ 
          7 │ fragment outerHouseTrainedFragment on Dog {
          8 │     ...isHouseTrainedFragment
          9 │ }
         10 │ 
         11 │ fragment isHouseTrainedFragment on Dog {
         12 │     isHouseTrained(atOtherHomes: $atOtherHomes)
            ·     ──────┬───────
            ·           ╰── Variable `atOtherHomes` is used here but not defined anywhere.
            ·
         13 │ }
            ╰────
        "#,
        )
    }

    #[test]
    fn test_all_variable_uses_defined_175() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        query houseTrainedQueryOne($atOtherHomes: Boolean) {
            dog {
                ...isHouseTrainedFragment
            }
        }

        query houseTrainedQueryTwo($atOtherHomes: Boolean) {
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
    fn test_all_variable_uses_defined_176() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        query houseTrainedQueryOne($atOtherHomes: Boolean) {
            dog {
                ...isHouseTrainedFragment
            }
        }

        query houseTrainedQueryTwoNotDefined {
            dog {
                ...isHouseTrainedFragment
            }
        }

        fragment isHouseTrainedFragment on Dog {
            isHouseTrained(atOtherHomes: $atOtherHomes)
        }
        "#,
            r#"
        Error: 5.8.3 All Variable Uses Defined
        
          × Variable `atOtherHomes` is not defined.
            ╭────
          1 │ query houseTrainedQueryOne($atOtherHomes: Boolean) {
          2 │     dog {
          3 │         ...isHouseTrainedFragment
          4 │     }
          5 │ }
          6 │ 
          7 │ query houseTrainedQueryTwoNotDefined {
          8 │     dog {
          9 │         ...isHouseTrainedFragment
         10 │     }
         11 │ }
         12 │ 
         13 │ fragment isHouseTrainedFragment on Dog {
         14 │     isHouseTrained(atOtherHomes: $atOtherHomes)
            ·     ──────┬───────
            ·           ╰── Variable `atOtherHomes` is used here but not defined anywhere.
            ·
         15 │ }
            ╰────
        "#,
        )
    }
}

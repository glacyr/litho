use graphql_parser::query::Text;

use crate::{Error, Traverse, Visitor};

/// # 5.1.1 Executable Definitions
///
/// ## Formal Specification
/// - For each definition `definition` in the document.
/// - `definition` must be `ExecutableDefinition` (it must not be
///   `TypeSystemDefinitionOrExtension`).
///
/// ## Explanatory Text
/// GraphQL execution will only consider the executable definitions Operation
/// and Fragment. Type system definitions and extensions are not executable, and
/// are not considered during execution.
///
/// To avoid ambiguity, a document containing `TypeSystemDefinitionOrExtension`
/// is invalid for execution.
///
/// GraphQL documents not intended to be directly executed may include
/// `TypeSystemDefinitionOrExtension`.
///
/// For example, the following document is invalid for execution since the
/// original executing schema may not know about the provided type extension:
///
/// ```graphql
/// query getDogName {
///   dog {
///     name
///     color
///   }
/// }
///
/// extend type Dog {
///   color: String
/// }
/// ```
pub struct ExecutableDefinitions;

impl<'v, 'a, T> Visitor<'v, 'a, T> for ExecutableDefinitions
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for ExecutableDefinitions
where
    'a: 'v,
    T: Text<'a>,
{
}

use graphql_parser::schema::Text;

use crate::{Error, Traverse, Visitor};

/// # 5.6.3 Input Object Field Uniqueness
/// ## Formal Specification
/// - For each input object value `inputObject` in the document.
/// - For every `inputField` in `inputObject`
///   - Let `name` be the Name of `inputField`.
///   - Let `fields` be all Input Object Fields named `name` in `inputObject`.
///   - `fields` must be the set containing only `inputField`.
///
/// ## Explanatory Text
/// Input objects must not contain more than one field of the same name,
/// otherwise an ambiguity would exist which includes an ignored portion of
/// syntax.
///
/// For example the following document will not pass validation.
/// ```graphql
/// {
///   field(arg: { field: true, field: false })
/// }
/// ```
pub struct InputObjectFieldUniqueness;

impl<'v, 'a, T> Visitor<'v, 'a, T> for InputObjectFieldUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    // Taken care of by `graphql_parser`.
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for InputObjectFieldUniqueness
where
    'a: 'v,
    T: Text<'a>,
{
}

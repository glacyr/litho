use graphql_parser::query::Text;

mod arguments;
mod documents;
mod fields;
mod fragments;
mod operations;

pub use arguments::*;
pub use documents::*;
pub use fields::*;
pub use fragments::*;
pub use operations::*;

use crate::{Error, Traverse};

pub const fn validators<'v, 'a, T>() -> impl Traverse<'v, 'a, T, Accumulator = Vec<Error<'v, 'a, T>>>
where
    'a: 'v,
    T: Text<'a>,
{
    (
        ExecutableDefinitions,
        OperationNameUniqueness,
        LoneAnonymousOperation,
        FieldSelections,
        LeafFieldSelections,
        ArgumentNames,
        ArgumentUniqueness,
        RequiredArguments,
        FragmentNameUniqueness,
        FragmentSpreadTypeExistence,
        FragmentsOnCompositeTypes,
        FragmentsMustBeUsed,
    )
}

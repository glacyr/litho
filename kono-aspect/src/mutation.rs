use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Intermediate, Value};

use super::ObjectValue;

/// Implemented by aspects that can resolve mutations (fields on the `Mutation`
/// type).
pub trait Mutation {
    type Context;
    type Error;

    /// Should return a boolean that indicates whether this aspect can resolve
    /// the mutation with the given name. This controls whether the aspect
    /// resolver calls [`Mutation::mutate(field, ...)`](Mutation::mutate).
    #[allow(unused)]
    fn can_mutate(field: &str) -> bool {
        false
    }

    /// Should resolve a mutation with the given name with the given arguments.
    /// Note that will only be called if
    /// [`Mutation::can_mutate(field)`](Mutation::can_mutate) returns true.
    #[allow(unused)]
    fn mutate<'a>(
        field: &'a str,
        args: HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<ObjectValue>, Self::Error>>>> {
        unreachable!()
    }
}

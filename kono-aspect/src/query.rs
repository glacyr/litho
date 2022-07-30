use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Intermediate, Value};

use super::ObjectValue;

/// Implemented by aspects that can resolve queries (fields on the `Query`
/// type).
pub trait Query {
    type Context;
    type Error;
    type Environment;

    /// Should return a boolean that indicates whether this aspect can resolve
    /// the query with the given name. This controls whether the aspect resolver
    /// calls [`Query::can_query(field, ...)`](Query::can_query).
    #[allow(unused)]
    fn can_query(environment: &Self::Environment, field: &str, context: &Self::Context) -> bool {
        false
    }

    /// Should resolve a query with the given name with the given arguments.
    /// Note that will only be called if
    /// [`Query::can_query(field)`](Query::can_query) returns true.
    #[allow(unused)]
    fn query<'a>(
        environment: &'a Self::Environment,
        field: &'a str,
        args: HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<ObjectValue>, Self::Error>> + 'a>> {
        unreachable!()
    }
}

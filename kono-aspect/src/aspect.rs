use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Error, Intermediate, Typename, Value};

use super::ObjectValue;

pub trait Aspect: Typename {
    type Context: 'static;
    type Environment;
    type Error: Error;

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
        field: &'a str,
        args: HashMap<String, Value>,
        context: &'a Self::Context,
        environment: &'a Self::Environment,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<ObjectValue>, Self::Error>> + 'a>> {
        unreachable!()
    }

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

    #[allow(unused)]
    fn can_resolve_field(&self, field: &str) -> bool {
        false
    }

    #[allow(unused)]
    fn resolve_field<'a>(
        &'a self,
        field: &'a str,
        args: &'a HashMap<String, Value>,
        context: &'a Self::Context,
        environment: &'a Self::Environment,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<ObjectValue>, Self::Error>> + 'a>> {
        unreachable!()
    }
}

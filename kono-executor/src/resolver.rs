use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{Error, Intermediate, Root, Typename, Value};

/// Implemented by types that can resolve GraphQL fields, queries, mutations and
/// subscriptions.
pub trait Resolver {
    /// Should be the context that gets passed along to each resolver within a
    /// request. This is the context that's passed to
    /// [`Executor::execute_request`](super::Executor::execute_request).
    type Context;

    /// Should be the error type. This can be used to customize error formatting
    /// and behavior.
    type Error: Error;

    /// Should be the type of intermediate values (in case fields don't resolve
    /// to a scalar immediately and have a subselection). This should basically
    /// be an enum of your own types, or something like `Box<dyn Any>`.
    type Value: Root + Typename;

    /// Should return a boolean that indicates if the given field can be
    /// resolved. This controls whether the executor calls
    /// [`Resolver::resolve`].
    fn can_resolve(
        &self,
        object_value: &Self::Value,
        field_name: &str,
        context: &Self::Context,
    ) -> bool;

    /// Should resolve the given field on the given object value with the given
    /// arguments. Note that this method will only be called when
    /// [`Resolver::can_resolve`] returned true.
    fn resolve<'a>(
        &'a self,
        object_value: &'a Self::Value,
        field_name: &'a str,
        argument_values: &'a HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<Self::Value>, Self::Error>> + 'a>>;
}

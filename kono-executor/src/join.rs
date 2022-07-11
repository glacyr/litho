use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{Intermediate, Resolver, Value};

pub struct Join<L, R>(L, R)
where
    L: Resolver,
    R: Resolver<Context = L::Context, Error = L::Error, Value = L::Value>;

impl<L, R> Resolver for Join<L, R>
where
    L: Resolver,
    R: Resolver<Context = L::Context, Error = L::Error, Value = L::Value>,
{
    type Context = L::Context;
    type Error = L::Error;
    type Value = L::Value;

    fn can_resolve(
        &self,
        object_ty: (),
        object_value: &Self::Value,
        field_name: &str,
        context: &Self::Context,
    ) -> bool {
        self.0
            .can_resolve(object_ty, object_value, field_name, context)
            || self
                .1
                .can_resolve(object_ty, object_value, field_name, context)
    }

    fn resolve<'a>(
        &'a self,
        object_ty: (),
        object_value: &'a Self::Value,
        field_name: &'a str,
        argument_values: &'a HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<Self::Value>, Self::Error>> + 'a>> {
        Box::pin(async move {
            if self
                .0
                .can_resolve(object_ty, object_value, field_name, context)
            {
                self.0.resolve(
                    object_ty,
                    object_value,
                    field_name,
                    argument_values,
                    context,
                )
            } else {
                self.1.resolve(
                    object_ty,
                    object_value,
                    field_name,
                    argument_values,
                    context,
                )
            }
            .await
        })
    }
}

pub fn join<L, R>(left: L, right: R) -> Join<L, R>
where
    L: Resolver,
    R: Resolver<Context = L::Context, Error = L::Error, Value = L::Value>,
{
    Join(left, right)
}

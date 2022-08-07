use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Intermediate, Resolver, Typename, Value};
use kono_schema::{Item, Schema};

use super::{Mutation, ObjectValue, OutputType, Query, Reference, ResolveField};

pub trait Aspect:
    Typename
    + ResolveField
    + Query<
        Context = <Self as ResolveField>::Context,
        Error = <Self as ResolveField>::Error,
        Environment = <Self as Aspect>::Environment,
    > + Mutation<Context = <Self as ResolveField>::Context, Error = <Self as ResolveField>::Error>
{
    type Environment;

    fn fetch<'a>(
        reference: Reference,
        context: &'a <Self as ResolveField>::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Box<Self>, <Self as ResolveField>::Error>> + 'a>> {
        let _ = reference;
        let _ = context;

        todo!()
    }
}

pub struct AspectResolver<A>(<A as Aspect>::Environment)
where
    A: Aspect;

impl<A> Resolver for AspectResolver<A>
where
    A: Aspect + 'static,
{
    type Context = <A as ResolveField>::Context;
    type Error = <A as ResolveField>::Error;
    type Value = ObjectValue;

    fn can_resolve<'a>(
        &self,
        object_value: &Self::Value,
        field_name: &str,
        context: &Self::Context,
    ) -> bool {
        match object_value {
            ObjectValue::Query => A::can_query(&self.0, field_name, context),
            ObjectValue::Aspect(aspect) => aspect
                .as_any()
                .downcast_ref::<A>()
                .map(|aspect| aspect.can_resolve_field(field_name))
                .unwrap_or_default(),
            _ => false,
        }
    }

    fn resolve<'a>(
        &'a self,
        object_value: &'a Self::Value,
        field_name: &'a str,
        argument_values: &'a HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<Self::Value>, Self::Error>> + 'a>> {
        match object_value {
            ObjectValue::Query => {
                A::query(&self.0, field_name, argument_values.to_owned(), context)
            }
            ObjectValue::Aspect(aspect) => aspect
                .as_any()
                .downcast_ref::<A>()
                .unwrap()
                .resolve_field(field_name, argument_values, context),
            _ => unreachable!(),
        }
    }
}

impl<A> Schema for AspectResolver<A>
where
    A: Aspect + OutputType<<A as Aspect>::Environment>,
{
    fn schema(&self) -> Vec<Item> {
        A::schema(&self.0)
    }
}

pub trait AspectExt: Aspect + Sized {
    fn resolver() -> AspectResolver<Self>
    where
        <Self as Aspect>::Environment: Default;

    fn with_env(environment: <Self as Aspect>::Environment) -> AspectResolver<Self>;
}

impl<A> AspectExt for A
where
    A: Aspect,
{
    fn resolver() -> AspectResolver<Self>
    where
        <Self as Aspect>::Environment: Default,
    {
        Self::with_env(Default::default())
    }

    fn with_env(environment: <Self as Aspect>::Environment) -> AspectResolver<Self> {
        AspectResolver(environment)
    }
}

use std::any::Any;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result};

use kono_executor::{Root, Typename};

use super::{Aspect, Reference};

pub enum ObjectValue {
    Query,
    Mutation,
    Subscription,
    Aspect(Box<dyn AnyAspect>),
    Reference(Reference),
}

impl Root for ObjectValue {
    fn query() -> Self {
        Self::Query
    }

    fn mutation() -> Self {
        Self::Mutation
    }

    fn subscription() -> Self {
        Self::Subscription
    }
}

pub trait AnyAspect: Typename {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AnyAspect for T
where
    T: Aspect + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Debug for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut tuple = f.debug_tuple("ObjectValue");

        match self {
            ObjectValue::Mutation => tuple.field(&"Mutation"),
            ObjectValue::Query => tuple.field(&"Query"),
            ObjectValue::Subscription => tuple.field(&"Subscription"),
            ObjectValue::Aspect(aspect) => tuple.field(&aspect.typename()),
            ObjectValue::Reference(reference) => tuple.field(reference),
        }
        .finish()
    }
}

impl Typename for ObjectValue {
    fn typename(&self) -> Cow<str> {
        match self {
            ObjectValue::Mutation => "Mutation".into(),
            ObjectValue::Query => "Query".into(),
            ObjectValue::Subscription => "Subscription".into(),
            ObjectValue::Aspect(aspect) => aspect.typename(),
            ObjectValue::Reference(reference) => reference.ty.to_owned().into(),
        }
    }
}

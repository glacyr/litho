use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{Intermediate, Resolver, Value};

macro_rules! impl_tuple {
    ( $first:ident $($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<$first, $($name),+> Resolver for ($first, $($name),+)
        where
            $first: Resolver,
            $($name: Resolver<Context = $first::Context, Error = $first::Error, Value = $first::Value>,)+
        {
            type Context = $first::Context;
            type Error = $first::Error;
            type Value = $first::Value;

            fn can_resolve(
                &self,
                object_ty: (),
                object_value: &Self::Value,
                field_name: &str,
                context: &Self::Context,
            ) -> bool {
                let (ref $first, $(ref $name),+) = self;

                if $first.can_resolve(object_ty, object_value, field_name, context) {
                    return true
                }

                $(if $name.can_resolve(object_ty, object_value, field_name, context) {
                    return true
                })+

                false
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
                    let (ref $first, $(ref $name),+) = self;

                    if $first.can_resolve(object_ty, object_value, field_name, context) {
                        return $first.resolve(
                            object_ty,
                            object_value,
                            field_name,
                            argument_values,
                            context,
                        ).await
                    }

                    $(if $name.can_resolve(object_ty, object_value, field_name, context) {
                        return $name.resolve(
                            object_ty,
                            object_value,
                            field_name,
                            argument_values,
                            context,
                        ).await
                    })+

                    todo!()
                })
            }
        }
    }
}

impl_tuple!(A B);
impl_tuple!(A B C);
impl_tuple!(A B C D);
impl_tuple!(A B C D E);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J K L);

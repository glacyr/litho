use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{Error, Intermediate, Value};

pub trait Resolver {
    type Context;
    type Error: Error;
    type Value: std::fmt::Debug;

    fn can_resolve<'a>(
        &self,
        object_ty: (),
        object_value: &Self::Value,
        field_name: &str,
        context: &Self::Context,
    ) -> bool;

    fn resolve<'a>(
        &'a self,
        object_ty: (),
        object_value: &'a Self::Value,
        field_name: &'a str,
        argument_values: &'a HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<Self::Value>, Self::Error>> + 'a>>;
}

// pub trait IntoResolver {
//     type Resolver: Resolver;

//     fn into_resolver(self) -> Self::Resolver;
// }

// macro_rules! impl_tuple {
//     ( $($name:ident)+) => {
//         impl<$($name),+> IntoResolver for ($($name),+)
//         where
//             $($name: IntoResolver,)+
//         {
//             type Resolver = ($($name::Resolver),+);

//             fn into_resolver(self) -> Self::Resolver {
//                 let ($($name),+) = self;
//                 ($($name.into_resolver()),+)
//             }
//         }
//     }
// }

// impl_tuple!(A B);
// impl_tuple!(A B C);
// impl_tuple!(A B C D);
// impl_tuple!(A B C D E);
// impl_tuple!(A B C D E F);
// impl_tuple!(A B C D E F G);
// impl_tuple!(A B C D E F G H);
// impl_tuple!(A B C D E F G H I);
// impl_tuple!(A B C D E F G H I J);
// impl_tuple!(A B C D E F G H I J K);
// impl_tuple!(A B C D E F G H I J K L);

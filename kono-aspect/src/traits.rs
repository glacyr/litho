use std::any::Any;
use std::collections::HashMap;

use serde::Serialize;

use kono_executor::{Intermediate, Value};

use super::{Aspect, ObjectValue, ResolveField};

pub trait IntoIntermediate<C, E> {
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E>;
}

// impl<C, E, T> IntoIntermediate<C, E> for &T
// where
//     T: ToOwned,
//     T::Owned: IntoIntermediate<C, E>,
// {
//     fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E> {
//         self.to_owned().into_intermediate()
//     }
// }

impl<C, E, T> IntoIntermediate<C, E> for Result<T, E>
where
    T: IntoIntermediate<C, E> + 'static,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E> {
        self.and_then(|value| value.into_intermediate())
    }
}

// impl<T> IntoIntermediate<T::Context, T::Error> for T
// where
//     T: ResolveField + 'static,
// {
//     fn into_intermediate(
//         self,
//     ) -> Result<Intermediate<ObjectValue<T::Context, T::Error>>, T::Error> {
//         Ok(Intermediate::Object(ObjectValue::Aspect(Box::new(self))))
//     }
// }

// pub trait Type {
//     type Environment;

//     fn schema(schema: &Self::Environment) -> Schema;
// }

// pub struct Schema {
//     pub name: String,
// }

macro_rules! ty {
    ($($ident:tt)*) => {
        impl<C, E> IntoIntermediate<C, E> for $($ident)* {
            fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E> {
                Ok(Intermediate::Value(self.into()))
            }
        }
    };
}

ty!(bool);
ty!(u8);
ty!(u16);
ty!(u32);
ty!(u64);
ty!(usize);
ty!(i8);
ty!(i16);
ty!(i32);
ty!(i64);
ty!(isize);
ty!(f32);
ty!(f64);
ty!(String);

ty!(&str);

impl<C, E, T> IntoIntermediate<C, E> for Option<T>
where
    T: Into<Value>,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E> {
        Ok(Intermediate::Value(self.into()))
    }
}

impl<C, E, T> IntoIntermediate<C, E> for Vec<T>
where
    T: Into<Value>,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E> {
        Ok(Intermediate::Value(self.into()))
    }
}

impl<C, E, T> IntoIntermediate<C, E> for HashMap<String, T>
where
    T: Into<Value>,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue<C, E>>, E> {
        Ok(Intermediate::Value(self.into_iter().collect()))
    }
}

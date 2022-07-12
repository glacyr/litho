use std::collections::HashMap;

use kono_executor::{Intermediate, Value};

use super::ObjectValue;

pub trait IntoIntermediate<E> {
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue>, E>;
}

impl<E, T> IntoIntermediate<E> for Option<T>
where
    T: IntoIntermediate<E>,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue>, E> {
        match self {
            Some(value) => value.into_intermediate(),
            None => ().into_intermediate(),
        }
    }
}

impl<T, E> IntoIntermediate<E> for Vec<T>
where
    T: IntoIntermediate<E> + 'static,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue>, E> {
        Ok(Intermediate::Collection(
            self.into_iter()
                .map(IntoIntermediate::into_intermediate)
                .collect::<Result<Vec<_>, E>>()?,
        ))
    }
}

impl<T, E> IntoIntermediate<E> for Result<T, E>
where
    T: IntoIntermediate<E> + 'static,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue>, E> {
        self.and_then(|value| value.into_intermediate())
    }
}

macro_rules! ty {
    ($($ident:tt)*) => {
        impl<E> IntoIntermediate<E> for $($ident)* {
            fn into_intermediate(self) -> Result<Intermediate<ObjectValue>, E> {
                Ok(Intermediate::Value(self.into()))
            }
        }
    };
}

ty!(());
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

impl<E, T> IntoIntermediate<E> for HashMap<String, T>
where
    T: Into<Value>,
{
    fn into_intermediate(self) -> Result<Intermediate<ObjectValue>, E> {
        Ok(Intermediate::Value(self.into_iter().collect()))
    }
}

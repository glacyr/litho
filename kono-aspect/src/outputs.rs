use kono_executor::{Intermediate, Value};
use kono_schema::{Item, ItemScalar, Type};

use super::{Error, ObjectValue};

pub trait OutputType<Env> {
    fn ty(environment: &Env) -> Type;

    fn schema(environment: &Env) -> Vec<Item>;

    fn inline(environment: &Env) -> bool {
        let _ = environment;

        false
    }

    fn inline_schema(environment: &Env) -> Vec<Item> {
        match Self::inline(environment) {
            true => Self::schema(environment),
            false => vec![],
        }
    }

    fn into_intermediate(self, environment: &Env) -> Result<Intermediate<ObjectValue>, Error>;
}

impl<Env, T> OutputType<Env> for Result<T, Error>
where
    T: OutputType<Env>,
{
    fn ty(environment: &Env) -> Type {
        Type::Optional(Box::new(T::ty(environment)))
    }

    fn schema(environment: &Env) -> Vec<Item> {
        T::schema(environment)
    }

    fn into_intermediate(self, environment: &Env) -> Result<Intermediate<ObjectValue>, Error> {
        self.and_then(|value| value.into_intermediate(environment))
    }
}

impl<Env, T> OutputType<Env> for Option<T>
where
    T: OutputType<Env>,
{
    fn ty(environment: &Env) -> Type {
        Type::Optional(Box::new(T::ty(environment)))
    }

    fn schema(environment: &Env) -> Vec<Item> {
        T::schema(environment)
    }

    fn into_intermediate(self, environment: &Env) -> Result<Intermediate<ObjectValue>, Error> {
        match self {
            Some(value) => value.into_intermediate(environment),
            None => ().into_intermediate(environment),
        }
    }
}

impl<Env, T> OutputType<Env> for Vec<T>
where
    T: OutputType<Env>,
{
    fn ty(environment: &Env) -> Type {
        Type::List(Box::new(T::ty(environment)))
    }

    fn schema(environment: &Env) -> Vec<Item> {
        T::schema(environment)
    }

    fn into_intermediate(self, environment: &Env) -> Result<Intermediate<ObjectValue>, Error> {
        Ok(Intermediate::Collection(
            self.into_iter()
                .map(|value| value.into_intermediate(environment))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<Env> OutputType<Env> for () {
    fn ty(environment: &Env) -> Type {
        Option::<bool>::ty(environment)
    }

    fn schema(environment: &Env) -> Vec<Item> {
        Option::<bool>::schema(environment)
    }

    fn into_intermediate(self, _environment: &Env) -> Result<Intermediate<ObjectValue>, Error> {
        Ok(Intermediate::Value(Value::Null))
    }
}

macro_rules! ty {
    ($ty:ty, $name:literal) => {
        impl<Env> OutputType<Env> for $ty {
            fn ty(_environment: &Env) -> Type {
                Type::Scalar($name.to_owned())
            }

            fn schema(_environment: &Env) -> Vec<Item> {
                vec![]
            }

            fn into_intermediate(
                self,
                _environment: &Env,
            ) -> Result<Intermediate<ObjectValue>, Error> {
                Ok(Intermediate::Value(self.into()))
            }
        }
    };
}

ty!(&str, "String");
ty!(String, "String");
ty!(bool, "Boolean");
ty!(u8, "Int");
ty!(u16, "Int");
ty!(u32, "Int");
ty!(u64, "Int");
ty!(i8, "Int");
ty!(i16, "Int");
ty!(i32, "Int");
ty!(i64, "Int");
ty!(f32, "Float");
ty!(f64, "Float");

use std::convert::{TryFrom, TryInto};

use kono_executor::{Error, Value};
use kono_schema::{Item, Type};

pub trait InputType<Env> {
    fn ty(environment: &Env) -> Type;

    fn schema(environment: &Env) -> Vec<Item>;

    fn from_value<E>(value: Value, environment: &Env) -> Result<Self, E>
    where
        Self: Sized,
        E: Error;

    fn from_value_option<E>(value: Option<Value>, environment: &Env) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Some(value) => Self::from_value(value, environment),
            None => Err(E::missing_value()),
        }
    }
}

macro_rules! ty {
    ($ty:ty, $name:literal, $($from:tt)*) => {
        impl<Env> InputType<Env> for $ty {
            fn ty(_environment: &Env) -> Type {
                Type::Scalar($name.to_owned())
            }

            fn schema(_environment: &Env) -> Vec<Item> {
				vec![]
            }

            fn from_value<E>(value: Value, _environment: &Env) -> Result<Self, E>
            where
                Self: Sized,
                E: Error,
            {
                match value {
                    $($from)*,
                    _ => Err(E::unexpected_value_type($name)),
                }
            }
        }
    };
}

trait AsNumber<T> {
    fn as_number(self) -> Option<T>;
}

impl AsNumber<u64> for serde_json::Number {
    fn as_number(self) -> Option<u64> {
        self.as_u64()
    }
}

impl AsNumber<i64> for serde_json::Number {
    fn as_number(self) -> Option<i64> {
        self.as_i64()
    }
}

impl AsNumber<f64> for serde_json::Number {
    fn as_number(self) -> Option<f64> {
        self.as_f64()
    }
}

trait FromNumber<T>: TryFrom<T>
where
    serde_json::Number: AsNumber<T>,
{
    fn from_number<E>(value: serde_json::Number) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        value
            .as_number()
            .ok_or(E::custom("can't represent number"))?
            .try_into()
            .map_err(|_| E::custom("can't represent number"))
    }
}

impl<T, U> FromNumber<U> for T
where
    T: TryFrom<U>,
    serde_json::Number: AsNumber<U>,
{
}

ty!(String, "String", Value::String(value) => Ok(value));
ty!(bool, "Boolean", Value::Bool(value) => Ok(value));
ty!(u8, "Int", Value::Number(value) => FromNumber::<u64>::from_number(value));
ty!(u16, "Int", Value::Number(value) => FromNumber::<u64>::from_number(value));
ty!(u32, "Int", Value::Number(value) => FromNumber::<u64>::from_number(value));
ty!(u64, "Int", Value::Number(value) => FromNumber::<u64>::from_number(value));
ty!(u128, "Int", Value::Number(value) => FromNumber::<u64>::from_number(value));
ty!(usize, "Int", Value::Number(value) => FromNumber::<u64>::from_number(value));
ty!(i8, "Int", Value::Number(value) => FromNumber::<i64>::from_number(value));
ty!(i16, "Int", Value::Number(value) => FromNumber::<i64>::from_number(value));
ty!(i32, "Int", Value::Number(value) => FromNumber::<i64>::from_number(value));
ty!(i64, "Int", Value::Number(value) => FromNumber::<i64>::from_number(value));
ty!(i128, "Int", Value::Number(value) => FromNumber::<i64>::from_number(value));
ty!(isize, "Int", Value::Number(value) => FromNumber::<i64>::from_number(value));
ty!(f32, "Float", Value::Number(value) => FromNumber::<f64>::from_number(value).map(|value: f64| value as f32));
ty!(f64, "Float", Value::Number(value) => FromNumber::<f64>::from_number(value));

impl<Env, T> InputType<Env> for Option<T>
where
    T: InputType<Env>,
{
    fn ty(environment: &Env) -> Type {
        Type::Optional(Box::new(T::ty(environment)))
    }

    fn schema(environment: &Env) -> Vec<Item> {
        T::schema(environment)
    }

    fn from_value<E>(value: Value, environment: &Env) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Value::Null => Ok(None),
            value => T::from_value(value, environment).map(Some),
        }
    }

    fn from_value_option<E>(value: Option<Value>, environment: &Env) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Some(value) => Self::from_value(value, environment),
            None => Ok(None),
        }
    }
}

impl<Env, T> InputType<Env> for Vec<T>
where
    T: InputType<Env>,
{
    fn ty(environment: &Env) -> Type {
        Type::List(Box::new(T::ty(environment)))
    }

    fn schema(environment: &Env) -> Vec<Item> {
        T::schema(environment)
    }

    fn from_value<E>(value: Value, environment: &Env) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Value::Array(items) => items
                .into_iter()
                .map(|value| T::from_value(value, environment))
                .collect(),
            _ => Err(E::custom("expected array")),
        }
    }
}

use kono_executor::{Error, Value};
use kono_schema::{Item, ItemScalar, Type};

pub trait InputType<Env> {
    fn ty(environment: &Env) -> Type;

    fn schema(environment: &Env) -> Vec<Item>;

    fn from_value<E>(value: Value) -> Result<Self, E>
    where
        Self: Sized,
        E: Error;

    fn from_value_option<E>(value: Option<Value>) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Some(value) => Self::from_value(value),
            None => Err(E::missing_value()),
        }
    }
}

pub trait OutputType<E> {
    fn ty(environment: &E) -> Type;

    fn schema(environment: &E) -> Vec<Item>;

    fn inline(environment: &E) -> bool {
        let _ = environment;

        false
    }

    fn inline_schema(environment: &E) -> Vec<Item> {
        match Self::inline(environment) {
            true => Self::schema(environment),
            false => vec![],
        }
    }
}

impl<E, T> OutputType<E> for Option<T>
where
    T: OutputType<E>,
{
    fn ty(environment: &E) -> Type {
        Type::Optional(Box::new(T::ty(environment)))
    }

    fn schema(environment: &E) -> Vec<Item> {
        T::schema(environment)
    }
}

impl<E, T> OutputType<E> for Vec<T>
where
    T: OutputType<E>,
{
    fn ty(environment: &E) -> Type {
        Type::List(Box::new(T::ty(environment)))
    }

    fn schema(environment: &E) -> Vec<Item> {
        T::schema(environment)
    }
}

impl<E> OutputType<E> for () {
    fn ty(environment: &E) -> Type {
        Option::<bool>::ty(environment)
    }

    fn schema(environment: &E) -> Vec<Item> {
        Option::<bool>::schema(environment)
    }
}

trait BuiltinType {
    fn from_value<E>(value: Value) -> Result<Self, E>
    where
        Self: Sized,
        E: Error;

    fn from_value_option<E>(value: Option<Value>) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Some(value) => Self::from_value(value),
            None => Err(E::custom("abcd")),
        }
    }
}

impl BuiltinType for String {
    fn from_value<E>(value: Value) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Value::String(string) => Ok(string),
            _ => Err(E::custom("abc")),
        }
    }
}

macro_rules! input {
    ($ty:ty, $name:literal) => {
        impl<Env> InputType<Env> for $ty {
            fn ty(_environment: &Env) -> Type {
                Type::Scalar($name.to_owned())
            }

            fn schema(_environment: &Env) -> Vec<Item> {
                vec![Item::Scalar(ItemScalar::new($name))]
            }

            fn from_value<E>(value: Value) -> Result<Self, E>
            where
                Self: Sized,
                E: Error,
            {
                BuiltinType::from_value(value)
            }
        }
    };
}

macro_rules! output {
    ($ty:ty, $name:literal) => {
        impl<E> OutputType<E> for $ty {
            fn ty(_environment: &E) -> Type {
                Type::Scalar($name.to_owned())
            }

            fn schema(_environment: &E) -> Vec<Item> {
                vec![Item::Scalar(ItemScalar::new($name))]
            }
        }
    };
}

impl<Env, T> InputType<Env> for Option<T>
where
    T: InputType<Env>,
{
    fn ty(environment: &Env) -> Type {
        T::ty(environment)
    }

    fn schema(environment: &Env) -> Vec<Item> {
        T::schema(environment)
    }

    fn from_value<E>(value: Value) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Value::Null => Ok(None),
            value => T::from_value(value).map(Some),
        }
    }

    fn from_value_option<E>(value: Option<Value>) -> Result<Self, E>
    where
        Self: Sized,
        E: Error,
    {
        match value {
            Some(value) => Self::from_value(value),
            None => Ok(None),
        }
    }
}

macro_rules! builtin {
    ($ty:ty, $name:literal) => {
        input!($ty, $name);
        output!($ty, $name);
    };
}

output!(&str, "String");
builtin!(String, "String");
output!(bool, "Boolean");
output!(u8, "Int");
output!(u16, "Int");
output!(u32, "Int");
output!(u64, "Int");
output!(i8, "Int");
output!(i16, "Int");
output!(i32, "Int");
output!(i64, "Int");
output!(f32, "Float");
output!(f64, "Float");

use kono_schema::{Item, ItemScalar, Type};

pub trait InputType<E> {
    fn ty(environment: &E) -> Type;

    fn schema(environment: &E) -> Vec<Item>;
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

macro_rules! builtin {
    ($ty:ty, $name:literal) => {
        impl<E> InputType<E> for $ty {
            fn ty(_environment: &E) -> Type {
                Type::Scalar($name.to_owned())
            }

            fn schema(_environment: &E) -> Vec<Item> {
                vec![Item::Scalar(ItemScalar::new($name))]
            }
        }

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

builtin!(&str, "String");
builtin!(String, "String");
builtin!(bool, "Boolean");
builtin!(u8, "Int");
builtin!(u16, "Int");
builtin!(u32, "Int");
builtin!(u64, "Int");
builtin!(i8, "Int");
builtin!(i16, "Int");
builtin!(i32, "Int");
builtin!(i64, "Int");
builtin!(f32, "Float");
builtin!(f64, "Float");

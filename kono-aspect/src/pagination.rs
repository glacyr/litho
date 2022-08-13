use std::collections::HashMap;
use std::iter::once;

use copa::{Connection, Edge, PageInfo, Pagination};
use kono_executor::{Intermediate, Value};
use kono_schema::{Field, Fields, InputValue, Item, ItemType, Type};

use super::{Error, ObjectValue, Record};

mod inputs {
    use super::*;

    use crate::{ArgumentType, InputType};

    impl<Env> ArgumentType<Env> for Pagination {
        fn schema(environment: &Env) -> Vec<InputValue> {
            vec![
                InputValue::new("first", Option::<usize>::ty(environment)),
                InputValue::new("last", Option::<usize>::ty(environment)),
                InputValue::new("after", Option::<String>::ty(environment)),
                InputValue::new("before", Option::<String>::ty(environment)),
            ]
        }

        fn from_args<E>(args: &HashMap<String, Value>, environment: &Env) -> Result<Self, E>
        where
            Self: Sized,
            E: kono_executor::Error,
        {
            let first =
                Option::<usize>::from_value_option(args.get("first").cloned(), environment)?;
            let last = Option::<usize>::from_value_option(args.get("last").cloned(), environment)?;
            let after =
                Option::<String>::from_value_option(args.get("after").cloned(), environment)?;
            let before =
                Option::<String>::from_value_option(args.get("before").cloned(), environment)?;

            match (first, last, after, before) {
                (Some(_), Some(_), _, _) => {
                    Err(E::custom("`first` and `last` are mutually exclusive"))
                }
                (_, _, Some(_), Some(_)) => {
                    Err(E::custom("`after` and `before` are mutually exclusive"))
                }
                (Some(_), _, _, Some(_)) => Err(E::custom(
                    "`before` cannot be present when `first` is present",
                )),
                (_, Some(_), Some(_), _) => Err(E::custom(
                    "`after` cannot be present when `last` is present",
                )),
                (Some(first), _, after, _) => Ok(Pagination::Forward { first, after }),
                (_, Some(last), _, before) => Ok(Pagination::Backward { last, before }),
                (_, last, _, Some(before)) => Ok(Pagination::Backward {
                    last: last.unwrap_or(10),
                    before: Some(before),
                }),
                (first, _, after, _) => Ok(Pagination::Forward {
                    first: first.unwrap_or(10),
                    after,
                }),
            }
        }
    }
}

mod outputs {
    use super::*;

    use crate::OutputType;

    macro_rules! insert {
        ($fields:ident, $name:literal, $value:expr, $env:ident) => {
            $fields.insert($name.to_owned(), $value.into_intermediate($env)?);
        };
    }

    impl<Env, T> OutputType<Env> for Connection<T>
    where
        T: OutputType<Env>,
    {
        fn ty(env: &Env) -> Type {
            Type::Named(format!(
                "{}Connection",
                T::ty(env).name().trim_end_matches("Edge")
            ))
        }

        fn inline(_environment: &Env) -> bool {
            true
        }

        fn schema(env: &Env) -> Vec<Item> {
            once(
                ItemType::new(Self::ty(env).name())
                    .fields(Fields::Named(vec![
                        Field::new(Some("edges"), Vec::<T>::ty(env)),
                        Field::new(Some("pageInfo"), PageInfo::ty(env)),
                    ]))
                    .into(),
            )
            .chain(T::inline_schema(env))
            .chain(PageInfo::inline_schema(env))
            .into_iter()
            .collect()
        }

        fn into_intermediate(self, env: &Env) -> Result<Intermediate<ObjectValue>, Error> {
            let mut fields = HashMap::new();
            insert!(fields, "edges", self.edges, env);
            insert!(fields, "pageInfo", self.page_info, env);
            Record::new(Self::ty(env).name(), fields).into_intermediate()
        }
    }

    impl<Env, T> OutputType<Env> for Edge<T>
    where
        T: OutputType<Env>,
    {
        fn ty(env: &Env) -> Type {
            Type::Named(format!("{}Edge", T::ty(env).name()))
        }

        fn inline(_environment: &Env) -> bool {
            true
        }

        fn schema(env: &Env) -> Vec<Item> {
            once(
                ItemType::new(Self::ty(env).name())
                    .fields(Fields::Named(vec![
                        Field::new(Some("node"), T::ty(env)),
                        Field::new(Some("cursor"), String::ty(env)),
                    ]))
                    .into(),
            )
            .chain(T::inline_schema(env))
            .into_iter()
            .collect()
        }

        fn into_intermediate(self, env: &Env) -> Result<Intermediate<ObjectValue>, Error> {
            let mut fields = HashMap::new();
            insert!(fields, "node", self.node, env);
            insert!(fields, "cursor", self.cursor, env);
            Record::new(Self::ty(env).name(), fields).into_intermediate()
        }
    }

    impl<Env> OutputType<Env> for PageInfo {
        fn ty(_environment: &Env) -> Type {
            Type::Named("PageInfo".to_owned())
        }

        fn inline(_environment: &Env) -> bool {
            true
        }

        fn schema(env: &Env) -> Vec<Item> {
            vec![ItemType::new("PageInfo")
                .fields(Fields::Named(vec![
                    Field::new(Some("hasPreviousPage"), bool::ty(env)),
                    Field::new(Some("hasNextPage"), bool::ty(env)),
                    Field::new(Some("startCursor"), Option::<String>::ty(env)),
                    Field::new(Some("endCursor"), Option::<String>::ty(env)),
                ]))
                .into()]
        }

        fn into_intermediate(self, env: &Env) -> Result<Intermediate<ObjectValue>, Error> {
            let mut fields = HashMap::new();
            insert!(fields, "hasPreviousPage", self.has_previous_page, env);
            insert!(fields, "hasNextPage", self.has_next_page, env);
            insert!(fields, "startCursor", self.start_cursor, env);
            insert!(fields, "endCursor", self.end_cursor, env);
            Record::new("PageInfo", fields).into_intermediate()
        }
    }
}

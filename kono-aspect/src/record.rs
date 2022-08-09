use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::ready;
use std::marker::PhantomData;

use kono_executor::{Error, Intermediate, Resolver, Typename};

use crate::ObjectValue;

#[derive(Debug)]
pub struct Record {
    name: String,
    records: RefCell<HashMap<String, Intermediate<ObjectValue>>>,
}

impl Record {
    pub fn new(name: &str, records: HashMap<String, Intermediate<ObjectValue>>) -> Record {
        Record {
            name: name.to_owned(),
            records: RefCell::new(records),
        }
    }
}

impl Typename for Record {
    fn typename(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }
}

pub struct RecordResolver<C, E>(PhantomData<(C, E)>)
where
    E: Error;

impl<C, E> Default for RecordResolver<C, E>
where
    E: Error,
{
    fn default() -> Self {
        RecordResolver(PhantomData)
    }
}

impl<C, E> Resolver for RecordResolver<C, E>
where
    E: Error,
{
    type Context = C;

    type Error = E;

    type Value = ObjectValue;

    fn can_resolve(
        &self,
        object_value: &Self::Value,
        field_name: &str,
        _context: &Self::Context,
    ) -> bool {
        match object_value {
            ObjectValue::Record(record) => record.records.borrow().contains_key(field_name),
            _ => false,
        }
    }

    fn resolve<'a>(
        &'a self,
        object_value: &'a Self::Value,
        field_name: &'a str,
        _argument_values: &'a std::collections::HashMap<String, kono_executor::Value>,
        _context: &'a Self::Context,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<kono_executor::Intermediate<Self::Value>, Self::Error>,
                > + 'a,
        >,
    > {
        match object_value {
            ObjectValue::Record(record) => Box::pin(ready(
                match record.records.borrow_mut().remove(field_name) {
                    Some(value) => Ok(value),
                    None => Err(E::unknown_field(&record.typename(), field_name)),
                },
            )),
            _ => unreachable!(),
        }
    }
}

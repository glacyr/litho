use std::any::Any;
use std::fmt::{Debug, Formatter, Result};

use super::Reference;

pub enum ObjectValue {
    Unit,
    Aspect(Box<dyn Any>),
    Reference(Reference),
}

impl Debug for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut tuple = f.debug_tuple("ObjectValue");

        match self {
            ObjectValue::Unit => tuple.field(&"Unit"),
            ObjectValue::Aspect(_) => tuple.field(&"Aspect"),
            ObjectValue::Reference(reference) => tuple.field(reference),
        }
        .finish()
    }
}

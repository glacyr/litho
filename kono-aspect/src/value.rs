use std::fmt::{Debug, Formatter, Result};

use super::{Reference, ResolveField};

pub enum ObjectValue<C, E> {
    Unit,
    Aspect(Box<dyn ResolveField<Context = C, Error = E>>),
    Reference(Reference),
}

impl<C, E> Debug for ObjectValue<C, E> {
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

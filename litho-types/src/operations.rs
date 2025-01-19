use std::hash::Hash;

use litho_language::ast::{ContextValue, OperationDefinition, Shared};

use super::Map;

#[derive(Debug)]
pub struct Operations<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub by_name: Map<'a, T, T, OperationDefinition<'a, T>>,
    pub nameless: Vec<Shared<'a, T, OperationDefinition<'a, T>>>,
}

impl<'a, T> Operations<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, OperationDefinition<'a, T>>> {
        self.by_name.get(name)
    }

    pub fn nameless(&self) -> impl Iterator<Item = &Shared<'a, T, OperationDefinition<'a, T>>> {
        self.nameless.iter()
    }

    pub fn len(&self) -> usize {
        self.by_name.len() + self.nameless.len()
    }
}

impl<'a, T> Default for Operations<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    fn default() -> Self {
        Operations {
            by_name: Default::default(),
            nameless: Default::default(),
        }
    }
}

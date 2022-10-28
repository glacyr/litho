use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::OperationDefinition;

use super::Map;

#[derive(Debug)]
pub struct Operations<T>
where
    T: Eq + Hash,
{
    pub by_name: Map<T, OperationDefinition<T>>,
    pub nameless: Vec<Arc<OperationDefinition<T>>>,
}

impl<T> Operations<T>
where
    T: Eq + Hash,
{
    pub fn by_name(&self, name: &T) -> impl Iterator<Item = &Arc<OperationDefinition<T>>> {
        self.by_name.get(name)
    }

    pub fn nameless(&self) -> impl Iterator<Item = &Arc<OperationDefinition<T>>> {
        self.nameless.iter()
    }

    pub fn len(&self) -> usize {
        self.by_name.len() + self.nameless.len()
    }
}

impl<T> Default for Operations<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Operations {
            by_name: Default::default(),
            nameless: Default::default(),
        }
    }
}

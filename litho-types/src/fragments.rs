use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::FragmentDefinition;

use super::Map;

#[derive(Debug)]
pub struct Fragments<T>
where
    T: Eq + Hash,
{
    pub by_name: Map<T, FragmentDefinition<T>>,
}

impl<T> Fragments<T>
where
    T: Eq + Hash,
{
    pub fn by_name(&self, name: &T) -> impl Iterator<Item = &Arc<FragmentDefinition<T>>> {
        self.by_name.get(name)
    }
}

impl<T> Default for Fragments<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Fragments {
            by_name: Default::default(),
        }
    }
}

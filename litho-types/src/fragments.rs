use std::hash::Hash;

use litho_language::ast::{ContextValue, FragmentDefinition, Shared};

use super::Map;

#[derive(Debug)]
pub struct Fragments<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub by_name: Map<'a, T, T, FragmentDefinition<'a, T>>,
}

impl<'a, T> Fragments<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, FragmentDefinition<'a, T>>> {
        self.by_name.get(name)
    }
}

impl<'a, T> Default for Fragments<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    fn default() -> Self {
        Fragments {
            by_name: Default::default(),
        }
    }
}

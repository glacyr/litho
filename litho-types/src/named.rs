use std::collections::HashMap;
use std::hash::Hash;

use litho_language::ast::{ContextValue, Shared};
use multimap::MultiMap;

#[derive(Debug)]
pub struct Named<'a, T, V>(HashMap<T, MultiMap<T, Shared<'a, T, V>>>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'a, T, V> Named<'a, T, V>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn all(&self) -> impl Iterator<Item = &Shared<'a, T, V>> {
        self.0
            .values()
            .flat_map(MultiMap::iter)
            .map(|(_, value)| value)
    }

    pub fn by_type(&self, ty: &T) -> impl Iterator<Item = &Shared<'a, T, V>> {
        self.0
            .get(ty)
            .into_iter()
            .flat_map(MultiMap::iter)
            .map(|(_, value)| value)
    }

    pub fn by_name(&self, ty: &T, name: &T) -> impl Iterator<Item = &Shared<'a, T, V>> {
        self.0
            .get(ty)
            .and_then(|map| map.get_vec(name))
            .into_iter()
            .flat_map(Vec::as_slice)
    }
}

impl<'a, T, V> Named<'a, T, V>
where
    T: ContextValue<'a> + Eq + Hash + ToOwned<Owned = T>,
{
    pub fn insert(&mut self, ty: &T, name: &T, value: &Shared<'a, T, V>) {
        self.0
            .entry(ty.to_owned())
            .or_default()
            .insert(name.to_owned(), value.to_owned());
    }
}

impl<'a, T, V> Default for Named<'a, T, V>
where
    T: ContextValue<'a> + Eq + Hash,
{
    fn default() -> Self {
        Named(Default::default())
    }
}

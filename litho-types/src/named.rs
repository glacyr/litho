use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use multimap::MultiMap;

#[derive(Debug)]
pub struct Named<T, V>(HashMap<T, MultiMap<T, Arc<V>>>)
where
    T: Eq + Hash;

impl<T, V> Named<T, V>
where
    T: Eq + Hash,
{
    pub fn all(&self) -> impl Iterator<Item = &Arc<V>> {
        self.0
            .values()
            .flat_map(MultiMap::iter)
            .map(|(_, value)| value)
    }

    pub fn by_type(&self, ty: &T) -> impl Iterator<Item = &Arc<V>> {
        self.0
            .get(ty)
            .into_iter()
            .flat_map(MultiMap::iter)
            .map(|(_, value)| value)
    }

    pub fn by_name(&self, ty: &T, name: &T) -> impl Iterator<Item = &Arc<V>> {
        self.0
            .get(ty)
            .and_then(|map| map.get_vec(name))
            .into_iter()
            .flat_map(Vec::as_slice)
    }
}

impl<T, V> Named<T, V>
where
    T: Eq + Hash + ToOwned<Owned = T>,
{
    pub fn insert(&mut self, ty: &T, name: &T, value: &Arc<V>) {
        self.0
            .entry(ty.to_owned())
            .or_default()
            .insert(name.to_owned(), value.to_owned());
    }
}

impl<T, V> Default for Named<T, V>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Named(Default::default())
    }
}

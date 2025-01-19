use std::hash::Hash;

use litho_language::ast::{ContextValue, Shared};
use multimap::MultiMap;

#[derive(Debug)]
pub struct Map<'a, T, K, V>(MultiMap<K, Shared<'a, T, V>>)
where
    T: ContextValue<'a>,
    K: Eq + Hash;

impl<'a, T, K, V> Map<'a, T, K, V>
where
    T: ContextValue<'a>,
    K: Eq + Hash,
{
    pub fn new() -> Map<'a, T, K, V> {
        Default::default()
    }

    pub fn insert(&mut self, key: &K, value: &Shared<'a, T, V>)
    where
        K: ToOwned<Owned = K>,
    {
        self.0.insert(key.to_owned(), value.clone());
    }

    pub fn get(&self, key: &K) -> impl Iterator<Item = &Shared<'a, T, V>> {
        self.0.get_vec(key).map(Vec::as_slice).into_iter().flatten()
    }

    pub fn len(&self) -> usize {
        self.0.iter().count()
    }
}

impl<'a, T, K, V> Default for Map<'a, T, K, V>
where
    T: ContextValue<'a>,
    K: Eq + Hash,
{
    fn default() -> Self {
        Map(MultiMap::new())
    }
}

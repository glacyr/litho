use std::hash::Hash;
use std::sync::Arc;

use multimap::MultiMap;

#[derive(Debug)]
pub struct Map<K, V>(MultiMap<K, Arc<V>>)
where
    K: Eq + Hash;

impl<K, V> Map<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Map<K, V> {
        Default::default()
    }

    pub fn insert(&mut self, key: &K, value: &Arc<V>)
    where
        K: ToOwned<Owned = K>,
    {
        self.0.insert(key.to_owned(), value.clone());
    }

    pub fn get(&self, key: &K) -> impl Iterator<Item = &Arc<V>> {
        self.0.get_vec(key).map(Vec::as_slice).into_iter().flatten()
    }
}

impl<K, V> Default for Map<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Map(MultiMap::new())
    }
}

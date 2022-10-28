use std::marker::PhantomData;
use std::sync::Arc;

use multimap::MultiMap;

#[derive(Debug)]
pub struct References<K, V>(MultiMap<usize, Arc<V>>, PhantomData<K>);

impl<K, V> References<K, V> {
    pub fn new() -> References<K, V> {
        Default::default()
    }

    fn key(node: &Arc<K>) -> usize {
        Arc::as_ptr(node) as usize
    }

    pub fn track(&mut self, node: &Arc<K>, usage: &Arc<V>) {
        self.0.insert(Self::key(node), usage.to_owned());
    }

    pub fn usages(&self, node: &Arc<K>) -> impl Iterator<Item = &Arc<V>> {
        self.0
            .get_vec(&Self::key(node))
            .into_iter()
            .flat_map(Vec::as_slice)
    }
}

impl<K, V> Default for References<K, V> {
    fn default() -> Self {
        References(Default::default(), Default::default())
    }
}

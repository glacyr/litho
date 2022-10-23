use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug)]
pub struct Inferred<K, V> {
    map: HashMap<usize, Arc<V>>,
    phantom: PhantomData<K>,
}

impl<K, V> Inferred<K, V> {
    fn key(&self, node: &Arc<K>) -> usize {
        Arc::as_ptr(node) as usize
    }

    pub fn get(&self, node: &Arc<K>) -> Option<&Arc<V>> {
        self.map.get(&self.key(node))
    }

    pub fn insert(&mut self, node: &Arc<K>, value: &Arc<V>) {
        self.map.insert(self.key(node), value.to_owned());
    }
}

impl<K, V> Default for Inferred<K, V> {
    fn default() -> Self {
        Inferred {
            map: Default::default(),
            phantom: Default::default(),
        }
    }
}

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::once;

#[derive(Debug)]
pub struct DepGraph<K, V>
where
    K: Eq + Hash,
    V: Eq + Hash,
{
    producers: HashMap<K, V>,
    consumers: HashMap<V, HashSet<K>>,
}

impl<K, V> DepGraph<K, V>
where
    K: Eq + Hash + Copy,
    V: Eq + Hash,
{
    pub fn new() -> DepGraph<K, V> {
        DepGraph {
            producers: Default::default(),
            consumers: Default::default(),
        }
    }

    pub fn produce(&mut self, producer: K, value: V) -> impl Iterator<Item = &K> {
        let consumers = self.consumers.get(&value).into_iter().flatten();
        self.producers.insert(producer, value);
        consumers
    }

    pub fn consume(&mut self, consumer: K, value: V) {
        self.consumers.entry(value).or_default().insert(consumer);
    }

    pub fn invalidate(&mut self, node: K, accumulator: &mut HashSet<K>) {
        if accumulator.contains(&node) {
            return;
        }

        let consumers = self
            .producers
            .get(&node)
            .and_then(|value| self.consumers.get(value))
            .cloned()
            .into_iter()
            .flatten();

        accumulator.extend(once(node));
        consumers
            .into_iter()
            .for_each(|node| self.invalidate(node, accumulator));
    }

    pub fn remove(&mut self, node: K) {
        self.producers.remove(&node);

        self.consumers.retain(|_, consumers| {
            consumers.remove(&node);
            !consumers.is_empty()
        });
    }
}

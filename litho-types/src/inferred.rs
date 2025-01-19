use std::collections::HashMap;
use std::marker::PhantomData;

use litho_language::ast::{AsPtr, ContextValue, Shared};

#[derive(Debug)]
pub struct Inferred<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    map: HashMap<usize, Shared<'a, T, V>>,
    phantom: PhantomData<K>,
}

impl<'a, T, K, V> Inferred<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn key(&self, node: &Shared<'a, T, K>) -> usize {
        node.as_ptr()
    }

    pub fn get(&self, node: &Shared<'a, T, K>) -> Option<&Shared<'a, T, V>> {
        self.map.get(&self.key(node))
    }

    pub fn insert(&mut self, node: &Shared<'a, T, K>, value: &Shared<'a, T, V>) {
        self.map.insert(self.key(node), value.to_owned());
    }
}

impl<'a, T, K, V> Default for Inferred<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn default() -> Self {
        Inferred {
            map: Default::default(),
            phantom: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct InferredSimple<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    map: HashMap<usize, V>,
    phantom: PhantomData<&'a (T, K)>,
}

impl<'a, T, K, V> InferredSimple<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn key(&self, node: &Shared<'a, T, K>) -> usize {
        node.as_ptr()
    }

    pub fn get(&self, node: &Shared<'a, T, K>) -> Option<&V> {
        self.map.get(&self.key(node))
    }

    pub fn insert(&mut self, node: &Shared<'a, T, K>, value: V) {
        self.map.insert(self.key(node), value);
    }
}

impl<'a, T, K, V> Default for InferredSimple<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn default() -> Self {
        InferredSimple {
            map: Default::default(),
            phantom: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct InferredMany<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    map: HashMap<usize, Vec<Shared<'a, T, V>>>,
    phantom: PhantomData<K>,
}

impl<'a, T, K, V> InferredMany<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn key(&self, node: &Shared<'a, T, K>) -> usize {
        node.as_ptr()
    }

    pub fn get(&self, node: &Shared<'a, T, K>) -> impl Iterator<Item = &Shared<'a, T, V>> {
        self.map.get(&self.key(node)).into_iter().flatten()
    }

    pub fn insert(&mut self, node: &Shared<'a, T, K>, value: &Shared<'a, T, V>) {
        self.map
            .entry(self.key(node))
            .or_default()
            .push(value.to_owned());
    }
}

impl<'a, T, K, V> Default for InferredMany<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn default() -> Self {
        InferredMany {
            map: Default::default(),
            phantom: Default::default(),
        }
    }
}

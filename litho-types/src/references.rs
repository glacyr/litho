use std::marker::PhantomData;

use litho_language::ast::{AsPtr, ContextValue, Shared};
use multimap::MultiMap;

#[derive(Debug)]
pub struct References<'a, T, K, V>(MultiMap<usize, Shared<'a, T, V>>, PhantomData<K>)
where
    T: ContextValue<'a>;

impl<'a, T, K, V> References<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    pub fn new() -> References<'a, T, K, V> {
        Default::default()
    }

    fn key(node: &Shared<'a, T, K>) -> usize {
        node.as_ptr()
    }

    pub fn track(&mut self, node: &Shared<'a, T, K>, usage: &Shared<'a, T, V>) {
        self.0.insert(Self::key(node), usage.to_owned());
    }

    pub fn usages(&self, node: &Shared<'a, T, K>) -> impl Iterator<Item = &Shared<'a, T, V>> {
        self.0
            .get_vec(&Self::key(node))
            .into_iter()
            .flat_map(Vec::as_slice)
    }
}

impl<'a, T, K, V> Default for References<'a, T, K, V>
where
    T: ContextValue<'a>,
{
    fn default() -> Self {
        References(Default::default(), Default::default())
    }
}

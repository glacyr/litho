use std::collections::HashMap;
use std::hash::Hash;

pub struct SourceMap<T> {
    map: HashMap<T, SourceId>,
    reverse_map: HashMap<SourceId, T>,
    next: usize,
}

impl<T> SourceMap<T> {
    pub fn new() -> SourceMap<T> {
        Default::default()
    }
}

impl<T> Default for SourceMap<T> {
    fn default() -> Self {
        SourceMap {
            map: Default::default(),
            reverse_map: Default::default(),
            next: 1,
        }
    }
}

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct SourceId(usize);

impl<T> SourceMap<T> {
    pub fn get_id(&self, id: &SourceId) -> Option<&T> {
        self.reverse_map.get(&id)
    }
}

impl<T> SourceMap<T>
where
    T: Eq + Hash,
{
    pub fn get(&self, key: &T) -> Option<SourceId> {
        self.map.get(key).cloned()
    }
}

impl<T> SourceMap<T>
where
    T: Clone + Eq + Hash,
{
    pub fn get_or_insert(&mut self, key: T) -> SourceId {
        let &mut id = self.map.entry(key.to_owned()).or_insert_with(|| {
            let result = self.next;
            self.next += 1;
            SourceId(result)
        });
        self.reverse_map.insert(id, key);
        id
    }
}

mod display {
    use std::fmt::{Debug, Display, Formatter, Result};

    use super::{SourceId, SourceMap};

    impl Debug for SourceId {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.write_str("SourceId(...)")
        }
    }

    impl Display for SourceId {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.write_str("SourceId(...)")
        }
    }

    impl<T> Debug for SourceMap<T>
    where
        T: Debug,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.reverse_map.fmt(f)
        }
    }

    impl<T> Display for SourceMap<T>
    where
        T: Debug,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.reverse_map.fmt(f)
        }
    }
}

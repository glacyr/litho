use std::hash::Hash;

use litho_language::ast::{FragmentDefinition, FragmentSpread};

use super::References;

#[derive(Debug)]
pub struct Usages<T>
where
    T: Eq + Hash,
{
    pub fragments: References<FragmentDefinition<T>, FragmentSpread<T>>,
}

impl<T> Default for Usages<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Usages {
            fragments: Default::default(),
        }
    }
}

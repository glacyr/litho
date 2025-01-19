use std::hash::Hash;

use litho_language::ast::{ContextValue, FragmentDefinition, FragmentSpread};

use super::References;

#[derive(Debug)]
pub struct Usages<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fragments: References<'a, T, FragmentDefinition<'a, T>, FragmentSpread<'a, T>>,
}

impl<'a, T> Default for Usages<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    fn default() -> Self {
        Usages {
            fragments: Default::default(),
        }
    }
}

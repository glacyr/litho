use std::hash::Hash;

use litho_language::ast::*;
use multimap::MultiMap;

use super::Named;

#[derive(Debug)]
pub struct Bindings<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub field_definitions: Named<'a, T, FieldDefinition<'a, T>>,
    pub input_value_definitions: Named<'a, T, InputValueDefinition<'a, T>>,
    pub enum_value_definitions: Named<'a, T, EnumValueDefinition<'a, T>>,
    pub union_member_types: Named<'a, T, NamedType<'a, T>>,
    pub schema_directives: Vec<Shared<'a, T, Directive<'a, T>>>,
    pub type_directives: MultiMap<T, Shared<'a, T, Directive<'a, T>>>,
}

impl<'a, T> Default for Bindings<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    fn default() -> Self {
        Bindings {
            field_definitions: Default::default(),
            input_value_definitions: Default::default(),
            enum_value_definitions: Default::default(),
            union_member_types: Default::default(),
            schema_directives: Default::default(),
            type_directives: Default::default(),
        }
    }
}

use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;

use super::Named;

#[derive(Debug)]
pub struct Bindings<T>
where
    T: Eq + Hash,
{
    pub field_definitions: Named<T, FieldDefinition<T>>,
    pub input_value_definitions: Named<T, InputValueDefinition<T>>,
    pub enum_value_definitions: Named<T, EnumValueDefinition<T>>,
    pub union_member_types: Named<T, NamedType<T>>,
    pub schema_directives: Vec<Arc<Directive<T>>>,
}

impl<T> Default for Bindings<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Bindings {
            field_definitions: Default::default(),
            input_value_definitions: Default::default(),
            enum_value_definitions: Default::default(),
            union_member_types: Default::default(),
            schema_directives: Default::default(),
        }
    }
}

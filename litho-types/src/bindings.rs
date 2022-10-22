use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use multimap::MultiMap;

#[derive(Debug)]
pub struct Bindings<T>
where
    T: Eq + Hash,
{
    pub(crate) field_definitions_by_type: MultiMap<T, Arc<FieldDefinition<T>>>,
    pub(crate) field_definitions_by_name: HashMap<T, MultiMap<T, Arc<FieldDefinition<T>>>>,
    pub(crate) enum_value_definitions: HashMap<T, MultiMap<T, Arc<EnumValueDefinition<T>>>>,
}

impl<T> Default for Bindings<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Bindings {
            field_definitions_by_type: Default::default(),
            field_definitions_by_name: Default::default(),
            enum_value_definitions: Default::default(),
        }
    }
}

impl<T> Bindings<T>
where
    T: Eq + Hash,
{
    /// Returns all field definitions of the given object or interface type.
    pub fn field_definitions_by_type(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Arc<FieldDefinition<T>>> {
        self.field_definitions_by_type
            .get_vec(ty)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }

    pub fn field_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Arc<FieldDefinition<T>>> {
        self.field_definitions_by_name
            .get(ty)
            .and_then(|ty| ty.get_vec(name))
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }

    pub fn enum_value_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Arc<EnumValueDefinition<T>>> {
        self.enum_value_definitions
            .get(ty)
            .into_iter()
            .flat_map(|ty| ty.iter().map(|(_, value)| value))
    }

    pub fn enum_value_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Arc<EnumValueDefinition<T>>> {
        self.enum_value_definitions
            .get(ty)
            .and_then(|ty| ty.get_vec(name))
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }
}

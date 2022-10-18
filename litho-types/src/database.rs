use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use multimap::MultiMap;

use super::index::Index;
use super::inference::{Inference, State};

#[derive(Debug)]
pub struct Database<T>
where
    T: Eq + Hash,
{
    pub(crate) operation_definitions_by_name: MultiMap<T, Arc<OperationDefinition<T>>>,
    pub(crate) fragment_definitions_by_name: MultiMap<T, Arc<FragmentDefinition<T>>>,
    pub(crate) directive_definitions_by_name: MultiMap<T, Arc<DirectiveDefinition<T>>>,
    pub(crate) type_definitions_by_name: MultiMap<T, Arc<TypeDefinition<T>>>,
    pub(crate) type_extensions_by_name: MultiMap<T, Arc<TypeExtension<T>>>,
    pub(crate) field_definitions: MultiMap<T, Arc<FieldDefinition<T>>>,
    pub(crate) field_definitions_by_field: MultiMap<usize, Arc<FieldDefinition<T>>>,
    pub(crate) field_definitions_by_name: HashMap<T, MultiMap<T, Arc<FieldDefinition<T>>>>,
    pub(crate) input_field_definitions: MultiMap<T, Arc<InputValueDefinition<T>>>,
    pub(crate) type_by_selection_set: HashMap<usize, T>,
    pub(crate) definition_for_arguments: HashMap<usize, Arc<ArgumentsDefinition<T>>>,
}

impl<T> Default for Database<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Database {
            operation_definitions_by_name: Default::default(),
            fragment_definitions_by_name: Default::default(),
            directive_definitions_by_name: Default::default(),
            type_definitions_by_name: Default::default(),
            type_extensions_by_name: Default::default(),
            field_definitions: Default::default(),
            field_definitions_by_field: Default::default(),
            field_definitions_by_name: Default::default(),
            input_field_definitions: Default::default(),
            type_by_selection_set: Default::default(),
            definition_for_arguments: Default::default(),
        }
    }
}

impl<T> Database<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Database<T> {
        Default::default()
    }
}

impl<T> Database<T>
where
    T: From<&'static str> + Clone + Debug + Eq + Hash,
{
    pub fn index(&mut self, document: &Document<T>) {
        document.traverse(&Index, self);
        document.traverse(&Inference, &mut State::new(self));
    }
}

impl<T> Database<T>
where
    T: Eq + Hash,
{
    pub fn type_definitions(&self) -> impl Iterator<Item = &TypeDefinition<T>> {
        self.type_definitions_by_name
            .iter_all()
            .flat_map(|(_, defs)| defs)
            .map(AsRef::as_ref)
    }

    pub fn type_definitions_by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Arc<TypeDefinition<T>>> {
        self.type_definitions_by_name
            .get_vec(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }

    pub fn directive_definitions_by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Arc<DirectiveDefinition<T>>> {
        self.directive_definitions_by_name
            .get_vec(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }

    pub fn is_input_type(&self, name: &T) -> bool {
        self.type_definitions_by_name(name)
            .any(|definition| definition.is_input())
    }

    pub fn is_output_type(&self, name: &T) -> bool {
        self.type_definitions_by_name(name)
            .any(|definition| definition.is_output())
    }

    pub fn is_object_type(&self, name: &T) -> bool {
        self.type_definitions_by_name(name)
            .any(|definition| definition.is_object_type())
    }

    pub fn is_union_member(&self, ty: &T, name: &T) -> bool {
        self.type_definitions_by_name(name)
            .any(|def| match def.as_ref() {
                TypeDefinition::UnionTypeDefinition(def) if def.includes_member_type(ty) => true,
                _ => false,
            })
    }

    pub fn implements_interface(&self, ty: &T, name: &T) -> bool {
        self.type_definitions_by_name(ty)
            .any(|def| match def.as_ref() {
                TypeDefinition::InterfaceTypeDefinition(def) if def.implements_interface(name) => {
                    true
                }
                TypeDefinition::ObjectTypeDefinition(def) if def.implements_interface(name) => true,
                _ => false,
            })
    }

    pub fn input_field_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &InputValueDefinition<T>> {
        self.input_field_definitions
            .get_vec(ty)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(AsRef::as_ref)
    }

    pub fn field_definitions(&self, ty: &T) -> impl Iterator<Item = &FieldDefinition<T>> {
        self.field_definitions
            .get_vec(ty)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(AsRef::as_ref)
    }

    pub fn field_definitions_by_field(
        &self,
        field: &Arc<Field<T>>,
    ) -> impl Iterator<Item = &FieldDefinition<T>> {
        self.field_definitions_by_field
            .get_vec(&(Arc::as_ptr(field) as usize))
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(AsRef::as_ref)
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

    pub fn type_by_selection_set(&self, selection_set: &Arc<SelectionSet<T>>) -> Option<&T> {
        self.type_by_selection_set
            .get(&(Arc::as_ptr(selection_set) as usize))
    }

    pub fn definition_for_arguments(
        &self,
        arguments: &Arc<Arguments<T>>,
    ) -> Option<&ArgumentsDefinition<T>> {
        self.definition_for_arguments
            .get(&(Arc::as_ptr(arguments) as usize))
            .map(AsRef::as_ref)
    }
}

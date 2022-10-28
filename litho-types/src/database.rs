use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use multimap::MultiMap;

use super::indexer::Indexer;
use super::inferencer::{InferenceState, Inferencer};
use super::{Bindings, Fragments, Inference, Operations};

#[derive(Debug)]
pub struct Database<T>
where
    T: Eq + Hash,
{
    pub definitions: Bindings<T>,
    pub extensions: Bindings<T>,
    pub inference: Inference<T>,
    pub operations: Operations<T>,
    pub fragments: Fragments<T>,
    pub(crate) directive_definitions_by_name: MultiMap<T, Arc<DirectiveDefinition<T>>>,
    pub(crate) type_definitions_by_name: MultiMap<T, Arc<TypeDefinition<T>>>,
    pub(crate) type_extensions_by_name: MultiMap<T, Arc<TypeExtension<T>>>,
}

impl<T> Default for Database<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Database {
            definitions: Default::default(),
            extensions: Default::default(),
            inference: Default::default(),
            operations: Default::default(),
            fragments: Default::default(),
            directive_definitions_by_name: Default::default(),
            type_definitions_by_name: Default::default(),
            type_extensions_by_name: Default::default(),
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

impl<'a, T> FromIterator<&'a Document<T>> for Database<T>
where
    T: From<&'static str> + Clone + Eq + Hash + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Document<T>>,
    {
        let mut database = Database::new();
        let docs = iter.into_iter().collect::<Vec<_>>();

        for document in docs.iter() {
            document.traverse(&Indexer, &mut database);
        }

        for document in docs.iter() {
            document.traverse(&Inferencer, &mut InferenceState::new(&mut database));
        }

        database
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

    pub fn type_extensions_by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Arc<TypeExtension<T>>> {
        self.type_extensions_by_name
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

    pub fn input_value_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Arc<InputValueDefinition<T>>> {
        self.definitions
            .input_value_definitions
            .by_type(ty)
            .chain(self.extensions.input_value_definitions.by_type(ty))
    }

    pub fn input_value_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Arc<InputValueDefinition<T>>> {
        self.definitions
            .input_value_definitions
            .by_name(ty, name)
            .chain(self.extensions.input_value_definitions.by_type(ty))
    }

    pub fn field_definitions(&self, ty: &T) -> impl Iterator<Item = &Arc<FieldDefinition<T>>> {
        self.definitions
            .field_definitions
            .by_type(ty)
            .chain(self.extensions.field_definitions.by_type(ty))
    }

    pub fn field_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Arc<FieldDefinition<T>>> {
        self.definitions
            .field_definitions
            .by_name(ty, name)
            .chain(self.extensions.field_definitions.by_name(ty, name))
    }

    pub fn implemented_interfaces(&self, ty: &T) -> impl Iterator<Item = &NamedType<T>> {
        let definitions = self
            .type_definitions_by_name(ty)
            .flat_map(|def| def.implements_interfaces());
        let extensions = self
            .type_extensions_by_name(ty)
            .flat_map(|def| def.implements_interfaces());

        definitions
            .chain(extensions)
            .flat_map(|def| def.named_types())
    }

    pub fn enum_value_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Arc<EnumValueDefinition<T>>> {
        self.definitions
            .enum_value_definitions
            .by_type(ty)
            .chain(self.extensions.enum_value_definitions.by_type(ty))
    }

    pub fn enum_value_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Arc<EnumValueDefinition<T>>> {
        self.definitions
            .enum_value_definitions
            .by_name(ty, name)
            .chain(self.extensions.enum_value_definitions.by_name(ty, name))
    }

    pub fn union_member_types(&self, ty: &T) -> impl Iterator<Item = &Arc<NamedType<T>>> {
        self.definitions
            .union_member_types
            .by_type(ty)
            .chain(self.extensions.union_member_types.by_type(ty))
    }

    pub fn union_member_types_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Arc<NamedType<T>>> {
        self.definitions
            .union_member_types
            .by_name(ty, name)
            .chain(self.extensions.union_member_types.by_name(ty, name))
    }
}

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use litho_language::ast::*;
use multimap::MultiMap;

use super::indexer::Indexer;
use super::inferencer::{InferenceState, Inferencer};
use super::{Bindings, Fragments, Import, Inference, Operations, Usages};

#[derive(Debug)]
pub struct Database<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub definitions: Bindings<'a, T>,
    pub extensions: Bindings<'a, T>,
    pub inference: Inference<'a, T>,
    pub operations: Operations<'a, T>,
    pub fragments: Fragments<'a, T>,
    pub usages: Usages<'a, T>,
    pub interface_implementations: MultiMap<T, T>,
    pub imports: HashMap<String, Import>,
    pub(crate) directive_definitions_by_name:
        MultiMap<T, Shared<'a, T, DirectiveDefinition<'a, T>>>,
    pub(crate) type_definitions_by_name: MultiMap<T, Shared<'a, T, TypeDefinition<'a, T>>>,
    pub(crate) type_extensions_by_name: MultiMap<T, Shared<'a, T, TypeExtension<'a, T>>>,
}

impl<'a, T> Default for Database<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    fn default() -> Self {
        Database {
            definitions: Default::default(),
            extensions: Default::default(),
            inference: Default::default(),
            operations: Default::default(),
            fragments: Default::default(),
            usages: Default::default(),
            interface_implementations: Default::default(),
            imports: Default::default(),
            directive_definitions_by_name: Default::default(),
            type_definitions_by_name: Default::default(),
            type_extensions_by_name: Default::default(),
        }
    }
}

impl<'a, T> Database<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn new() -> Database<'a, T> {
        Default::default()
    }

    pub fn imports(&self) -> &HashMap<String, Import> {
        &self.imports
    }

    pub fn with_imports<'b, I>(
        iter: I,
        imports: &HashMap<String, Result<&'b Document<'a, T>, String>>,
    ) -> Self
    where
        I: IntoIterator<Item = &'b Document<'a, T>>,
        T: Borrow<str> + From<&'a str> + ToString,
    {
        let mut database = Database::new();
        let docs = iter.into_iter().collect::<Vec<_>>();

        for document in docs.iter() {
            document.traverse(&Indexer, &mut database);
        }

        let directives = database
            .schema_directives()
            .flat_map(|directive| {
                let Some(name) = directive.name.ok() else {
                    return None;
                };

                if <T as Borrow<str>>::borrow(name.as_ref()) != "litho" {
                    return None;
                }

                let Some(url) = directive
                    .arguments
                    .iter()
                    .flat_map(|args| args.items.iter())
                    .find(|arg| <T as Borrow<str>>::borrow(arg.name.as_ref()) == "url")
                    .and_then(|arg| arg.value.ok())
                else {
                    return None;
                };

                let Value::StringValue(url) = url.as_ref() else {
                    return None;
                };

                let headers = match directive
                    .arguments
                    .iter()
                    .flat_map(|args| args.items.iter())
                    .find(|arg| <T as Borrow<str>>::borrow(arg.name.as_ref()) == "headers")
                    .and_then(|arg| arg.value.ok())
                {
                    Some(value) => value
                        .to_json()
                        .and_then(|json| serde_json::from_value(json).ok())
                        .unwrap_or_default(),
                    None => vec![],
                };

                let import = Import {
                    headers,
                    refresh: Duration::from_secs(60),
                };

                Some((url.to_string(), import))
            })
            .collect::<HashMap<_, _>>();

        for url in directives.keys() {
            let Some(Ok(document)) = imports.get(url) else {
                continue;
            };

            document.traverse(&Indexer, &mut database);
        }

        database.imports = directives;

        for document in docs.iter() {
            document.traverse(&Inferencer, &mut InferenceState::new(&mut database));
        }

        database
    }
}

impl<'a, 'b, T> FromIterator<&'b Document<'a, T>> for Database<'a, T>
where
    T: ContextValue<'a> + Borrow<str> + Eq + From<&'a str> + Hash + ToString,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'b Document<'a, T>>,
    {
        Database::with_imports(iter, &Default::default())
    }
}

impl<'a, T> Database<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn type_definitions(&self) -> impl Iterator<Item = &TypeDefinition<'a, T>> {
        self.type_definitions_by_name
            .iter_all()
            .flat_map(|(_, defs)| defs)
            .map(AsRef::as_ref)
    }

    pub fn type_definitions_by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, TypeDefinition<'a, T>>> {
        self.type_definitions_by_name
            .get_vec(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }

    pub fn type_extensions_by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, TypeExtension<'a, T>>> {
        self.type_extensions_by_name
            .get_vec(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
    }

    pub fn directive_definitions_by_name(
        &self,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, DirectiveDefinition<'a, T>>> {
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

    pub fn is_composite_type(&self, name: &T) -> bool {
        self.type_definitions_by_name(name)
            .any(|definition| definition.is_composite())
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

    pub fn interface_implementations(&self, interface: &T) -> impl Iterator<Item = &T> {
        self.interface_implementations
            .get_vec(interface)
            .into_iter()
            .flatten()
    }

    pub fn input_value_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, InputValueDefinition<'a, T>>> {
        self.definitions
            .input_value_definitions
            .by_type(ty)
            .chain(self.extensions.input_value_definitions.by_type(ty))
    }

    pub fn input_value_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, InputValueDefinition<'a, T>>> {
        self.definitions
            .input_value_definitions
            .by_name(ty, name)
            .chain(self.extensions.input_value_definitions.by_name(ty, name))
    }

    pub fn field_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, FieldDefinition<'a, T>>> {
        self.definitions
            .field_definitions
            .by_type(ty)
            .chain(self.extensions.field_definitions.by_type(ty))
    }

    pub fn field_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, FieldDefinition<'a, T>>> {
        self.definitions
            .field_definitions
            .by_name(ty, name)
            .chain(self.extensions.field_definitions.by_name(ty, name))
    }

    pub fn implemented_interfaces(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, NamedType<'a, T>>> {
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

    pub fn implemented_interfaces_by_name<'b>(
        &'b self,
        ty: &T,
        name: &'b T,
    ) -> impl Iterator<Item = &Shared<'a, T, NamedType<'a, T>>> + 'b {
        self.implemented_interfaces(ty)
            .filter(move |interface| interface.0.as_ref() == name)
    }

    pub fn enum_value_definitions(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, EnumValueDefinition<'a, T>>> {
        self.definitions
            .enum_value_definitions
            .by_type(ty)
            .chain(self.extensions.enum_value_definitions.by_type(ty))
    }

    pub fn enum_value_definitions_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, EnumValueDefinition<'a, T>>> {
        self.definitions
            .enum_value_definitions
            .by_name(ty, name)
            .chain(self.extensions.enum_value_definitions.by_name(ty, name))
    }

    pub fn union_member_types(
        &self,
        ty: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, NamedType<'a, T>>> {
        self.definitions
            .union_member_types
            .by_type(ty)
            .chain(self.extensions.union_member_types.by_type(ty))
    }

    pub fn union_member_types_by_name(
        &self,
        ty: &T,
        name: &T,
    ) -> impl Iterator<Item = &Shared<'a, T, NamedType<'a, T>>> {
        self.definitions
            .union_member_types
            .by_name(ty, name)
            .chain(self.extensions.union_member_types.by_name(ty, name))
    }

    pub fn type_exists(&self, ty: &T) -> bool {
        self.type_definitions_by_name(ty).next().is_some()
    }

    pub fn possible_types<'b>(&'b self, ty: &'b T) -> impl Iterator<Item = &T> + use<'a, 'b, T> {
        std::iter::once(ty)
            .chain(self.union_member_types(ty).map(|ty| ty.0.as_ref()))
            .chain(self.interface_implementations(ty))
    }

    pub fn schema_directives(&self) -> impl Iterator<Item = &Shared<'a, T, Directive<'a, T>>> {
        self.definitions
            .schema_directives
            .iter()
            .chain(self.extensions.schema_directives.iter())
    }

    fn both<'b, F, O>(&'b self, apply: F) -> impl Iterator<Item = O::Item> + 'b
    where
        F: Fn(&'b Bindings<'a, T>) -> O,
        O: Iterator + 'b,
    {
        apply(&self.definitions).chain(apply(&self.extensions))
    }

    pub fn type_directives<'b>(
        &'b self,
        ty: &'b T,
    ) -> impl Iterator<Item = &Shared<'a, T, Directive<'a, T>>> {
        self.both(move |bindings| bindings.type_directives.get_vec(ty).into_iter().flatten())
    }
}

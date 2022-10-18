use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;

use super::Database;

pub struct Index;

impl<'ast, T> Visit<'ast, T> for Index
where
    T: From<&'static str> + Clone + Eq + Hash + 'ast,
{
    type Accumulator = Database<T>;

    fn visit_directive_definition(
        &self,
        node: &'ast Arc<DirectiveDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            accumulator
                .directive_definitions_by_name
                .insert(name.as_ref().clone(), node.clone());
        }
    }

    fn visit_type_definition(
        &self,
        node: &'ast Arc<TypeDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name().ok() {
            accumulator
                .type_definitions_by_name
                .insert(name.as_ref().clone(), node.clone());
        }
    }

    fn visit_type_extension(
        &self,
        node: &'ast Arc<TypeExtension<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name().ok() {
            accumulator
                .type_extensions_by_name
                .insert(name.as_ref().clone(), node.clone());
        }
    }

    fn visit_object_type_definition(
        &self,
        node: &'ast ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator
                    .field_definitions
                    .insert(name.as_ref().clone(), field.clone());
                accumulator
                    .field_definitions_by_name
                    .entry(name.as_ref().clone())
                    .or_default()
                    .insert(field.name.as_ref().clone(), field.clone());
            }
        }
    }

    fn visit_object_type_extension(
        &self,
        node: &'ast ObjectTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator
                    .field_definitions
                    .insert(name.as_ref().clone(), field.clone());
                accumulator
                    .field_definitions_by_name
                    .entry(name.as_ref().clone())
                    .or_default()
                    .insert(field.name.as_ref().clone(), field.clone());
            }
        }
    }

    fn visit_interface_type_definition(
        &self,
        node: &'ast InterfaceTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator
                    .field_definitions
                    .insert(name.as_ref().clone(), field.clone());
                accumulator
                    .field_definitions_by_name
                    .entry(name.as_ref().clone())
                    .or_default()
                    .insert(field.name.as_ref().clone(), field.clone());
            }
        }
    }

    fn visit_interface_type_extension(
        &self,
        node: &'ast InterfaceTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator
                    .field_definitions
                    .insert(name.as_ref().clone(), field.clone());
                accumulator
                    .field_definitions_by_name
                    .entry(name.as_ref().clone())
                    .or_default()
                    .insert(field.name.as_ref().clone(), field.clone());
            }
        }
    }

    fn visit_input_object_type_definition(
        &self,
        node: &'ast InputObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator
                    .input_field_definitions
                    .insert(name.as_ref().clone(), field.clone());
            }
        }
    }
}

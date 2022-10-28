use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;

use super::Database;

pub struct Indexer;

impl<'ast, T> Visit<'ast, T> for Indexer
where
    T: 'ast + Clone + Eq + Hash,
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
        if let Some(name) = node.name() {
            accumulator
                .type_extensions_by_name
                .insert(name.clone(), node.clone());
        }
    }

    fn visit_object_type_definition(
        &self,
        node: &'ast ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator.definitions.field_definitions.insert(
                    name.as_ref(),
                    field.name.as_ref(),
                    field,
                );
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
                accumulator.extensions.field_definitions.insert(
                    name.0.as_ref(),
                    field.name.as_ref(),
                    field,
                );
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
                accumulator.definitions.field_definitions.insert(
                    name.as_ref(),
                    field.name.as_ref(),
                    field,
                );
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
                accumulator.extensions.field_definitions.insert(
                    name.0.as_ref(),
                    field.name.as_ref(),
                    field,
                );
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
                accumulator.definitions.input_value_definitions.insert(
                    name.as_ref(),
                    field.name.as_ref(),
                    field,
                );
            }
        }
    }

    fn visit_input_object_type_extension(
        &self,
        node: &'ast InputObjectTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, fields)) = node.name.ok().zip(node.fields_definition.as_ref()) {
            for field in fields.definitions.iter() {
                accumulator.extensions.input_value_definitions.insert(
                    name.0.as_ref(),
                    field.name.as_ref(),
                    field,
                );
            }
        }
    }

    fn visit_enum_type_definition(
        &self,
        node: &'ast EnumTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, values)) = node.name.ok().zip(node.values_definition.as_ref()) {
            for value in values.definitions.iter() {
                accumulator.definitions.enum_value_definitions.insert(
                    name.as_ref(),
                    value.enum_value.0.as_ref(),
                    value,
                );
            }
        }
    }

    fn visit_enum_type_extension(
        &self,
        node: &'ast EnumTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, values)) = node.name.ok().zip(node.values_definition.as_ref()) {
            for value in values.definitions.iter() {
                accumulator.extensions.enum_value_definitions.insert(
                    name.0.as_ref(),
                    value.enum_value.0.as_ref(),
                    value,
                );
            }
        }
    }

    fn visit_union_type_definition(
        &self,
        node: &'ast UnionTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, types)) = node.name.ok().zip(node.member_types.as_ref()) {
            for ty in types.named_types() {
                accumulator
                    .definitions
                    .union_member_types
                    .insert(name.as_ref(), ty.0.as_ref(), ty);
            }
        }
    }

    fn visit_union_type_extension(
        &self,
        node: &'ast UnionTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some((name, types)) = node.name.ok().zip(node.member_types.as_ref()) {
            for ty in types.named_types() {
                accumulator.extensions.union_member_types.insert(
                    name.0.as_ref(),
                    ty.0.as_ref(),
                    ty,
                );
            }
        }
    }

    fn visit_operation_definition(
        &self,
        node: &'ast Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.name.as_ref() {
            Some(name) => accumulator.operations.by_name.insert(name.as_ref(), node),
            None => accumulator.operations.nameless.push(node.clone()),
        }
    }
}

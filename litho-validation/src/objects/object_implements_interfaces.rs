use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::once;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct ObjectImplementsInterfaces<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> ObjectImplementsInterfaces<'a, T>
where
    T: Eq + Hash,
{
    fn is_valid_implementation(
        &self,
        interface_named_type: &'a NamedType<T>,
        implementation: &'a ObjectTypeDefinition<T>,
    ) -> Vec<Error<'a, T>> {
        let implementation_name = match implementation.name.ok() {
            Some(name) => name,
            None => return vec![],
        };

        let interface = match self
            .0
            .type_definitions_by_name(interface_named_type.0.as_ref())
            .next()
        {
            Some(&TypeDefinition::InterfaceTypeDefinition(ref definition)) => definition,
            Some(_) => {
                return vec![Error::ImplementsNonInterfaceType {
                    name: implementation_name.as_ref(),
                    interface: interface_named_type.0.as_ref(),
                    span: interface_named_type.span(),
                }]
            }
            None => return vec![],
        };

        for inherited in interface
            .implements_interfaces
            .iter()
            .flat_map(|implements| implements.types())
        {
            if !implementation.implements_interface(inherited) {
                return vec![Error::MissingInheritedInterface {
                    name: implementation_name.as_ref(),
                    interface: interface_named_type.0.as_ref(),
                    inherited,
                    span: interface_named_type.span(),
                }];
            }
        }

        let interface_fields = match interface.fields_definition.as_ref() {
            Some(interface_fields) => interface_fields,
            None => return vec![],
        };

        let implementation_fields = match implementation.fields_definition.as_ref() {
            Some(implementation_fields) => implementation_fields,
            None => return vec![],
        };

        let mut errors = vec![];

        for field in interface_fields.definitions.iter() {
            let implemented_field = match implementation_fields.field(field.name.as_ref()) {
                Some(field) => field,
                None => {
                    errors.push(Error::MissingInterfaceField {
                        name: implementation_name.as_ref(),
                        interface: interface_named_type.0.as_ref(),
                        field: field.name.as_ref(),
                        span: interface_named_type.span(),
                    });

                    continue;
                }
            };

            let mut visited = HashSet::new();

            for argument in field
                .arguments_definition
                .iter()
                .flat_map(|def| def.definitions.iter())
            {
                visited.insert(argument.name.as_ref());

                match implemented_field
                    .arguments_definition
                    .as_ref()
                    .and_then(|def| def.argument(argument.name.as_ref()))
                {
                    Some(implemented_argument) => {
                        match argument.ty.ok().zip(implemented_argument.ty.ok()) {
                            Some((expected, ty)) if !expected.is_invariant(ty) => {
                                errors.push(Error::InvalidInterfaceFieldArgumentType {
                                    name: implementation_name.as_ref(),
                                    interface: interface_named_type.0.as_ref(),
                                    field: field.name.as_ref(),
                                    argument: argument.name.as_ref(),
                                    expected,
                                    ty,
                                    span: ty.span(),
                                })
                            }
                            _ => {}
                        }
                    }
                    None => errors.push(Error::MissingInterfaceFieldArgument {
                        name: implementation_name.as_ref(),
                        interface: interface_named_type.0.as_ref(),
                        field: field.name.as_ref(),
                        argument: argument.name.as_ref(),
                        span: implemented_field.name.span(),
                    }),
                }
            }

            for implemented_argument in implemented_field
                .arguments_definition
                .iter()
                .flat_map(|def| def.definitions.iter())
            {
                if visited.contains(&implemented_argument.name.as_ref()) {
                    continue;
                }

                match implemented_argument.ty.ok() {
                    Some(ty) if ty.is_required() => {
                        errors.push(Error::UnexpectedNonNullInterfaceFieldArgument {
                            name: implementation_name.as_ref(),
                            interface: interface_named_type.0.as_ref(),
                            field: field.name.as_ref(),
                            argument: implemented_argument.name.as_ref(),
                            ty,
                            span: ty.span(),
                        })
                    }
                    _ => {}
                }
            }

            match field.ty.ok().zip(implemented_field.ty.ok()) {
                Some((field_type, implemented_field_type))
                    if !self
                        .is_valid_implementation_field_type(implemented_field_type, field_type) =>
                {
                    errors.push(Error::NonCovariantInterfaceField {
                        name: implementation_name.as_ref(),
                        interface: interface_named_type.0.as_ref(),
                        field: field.name.as_ref(),
                        expected: &field_type,
                        ty: &implemented_field_type,
                        span: implemented_field_type.span(),
                    })
                }
                _ => {}
            }
        }

        errors
    }

    pub fn is_valid_implementation_field_type(
        &self,
        field_type: &Type<T>,
        implemented_field_type: &Type<T>,
    ) -> bool {
        match (field_type, implemented_field_type) {
            (Type::NonNull(field_type), Type::NonNull(implemented_field_type)) => {
                self.is_valid_implementation_field_type(&field_type.ty, &implemented_field_type.ty)
            }
            (Type::NonNull(field_type), implemented_field_type) => {
                self.is_valid_implementation_field_type(&field_type.ty, implemented_field_type)
            }
            (Type::List(field_type), Type::List(implemented_field_type)) => {
                match field_type.ty.ok().zip(implemented_field_type.ty.ok()) {
                    Some((field_type, implemented_field_type)) => self
                        .is_valid_implementation_field_type(&field_type, &implemented_field_type),
                    None => true,
                }
            }
            (lhs, rhs) if lhs.is_invariant(rhs) => true,
            (Type::Named(lhs), Type::Named(rhs))
                if self.0.is_union_member(lhs.0.as_ref(), rhs.0.as_ref())
                    || self.0.implements_interface(lhs.0.as_ref(), rhs.0.as_ref()) =>
            {
                true
            }
            _ => false,
        }
    }
}

impl<'a, T> Visit<'a, T> for ObjectImplementsInterfaces<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let interfaces = match node.implements_interfaces.as_ref() {
            Some(interfaces) => interfaces,
            None => return,
        };

        let mut visited = HashMap::<&T, &NamedType<T>>::new();

        for interface in
            once(&interfaces.first).chain(interfaces.types.iter().map(|(_, interface)| interface))
        {
            let interface = match interface.ok() {
                Some(interface) => interface,
                _ => continue,
            };

            match visited.get(&interface.0.as_ref()) {
                Some(exists) => {
                    accumulator.push(Error::DuplicateImplementsInterface {
                        name: interface.0.as_ref(),
                        first: exists.span(),
                        second: interface.span(),
                    });
                    continue;
                }
                None => {}
            }

            visited.insert(interface.0.as_ref(), interface);

            accumulator.extend(self.is_valid_implementation(interface, node));
        }
    }
}

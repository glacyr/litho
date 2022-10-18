use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct ImplementsInterface<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> ImplementsInterface<'a, T>
where
    T: Eq + Hash,
{
    fn check_inherited_implementations(
        &self,
        interface_definition: &'a InterfaceTypeDefinition<T>,
        interface_named_type: &'a NamedType<T>,
        name: &'a T,
        implements_interfaces: &ImplementsInterfaces<T>,
    ) -> Option<Error<'a, T>> {
        for inherited in interface_definition
            .implements_interfaces
            .iter()
            .flat_map(|implements| implements.types())
        {
            if !implements_interfaces.implements_interface(inherited) {
                return Some(Error::MissingInheritedInterface {
                    name,
                    interface: interface_named_type.0.as_ref(),
                    inherited,
                    span: interface_named_type.span(),
                });
            }
        }

        None
    }

    fn check_valid_implementation(
        &self,
        interface_name: &'a NamedType<T>,
        interface: &'a InterfaceTypeDefinition<T>,
        concrete_name: &'a T,
        concrete: &'a TypeDefinition<T>,
    ) -> Vec<Error<'a, T>> {
        let interface_fields = match interface.fields_definition.as_ref() {
            Some(interface_fields) => interface_fields,
            None => return vec![],
        };

        let implementation_fields = match concrete.fields_definition() {
            Some(implementation_fields) => implementation_fields,
            None => return vec![],
        };

        let mut errors = vec![];

        for field in interface_fields.definitions.iter() {
            let implemented_field = match implementation_fields.field(field.name.as_ref()) {
                Some(field) => field,
                None => {
                    errors.push(Error::MissingInterfaceField {
                        name: concrete_name,
                        interface: interface_name.0.as_ref(),
                        field: field.name.as_ref(),
                        span: interface_name.span(),
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
                                    name: concrete_name,
                                    interface: interface_name.0.as_ref(),
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
                        name: concrete_name,
                        interface: interface_name.0.as_ref(),
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
                            name: concrete_name,
                            interface: interface_name.0.as_ref(),
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
                        name: concrete_name,
                        interface: interface_name.0.as_ref(),
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

    pub fn check_interface(
        &self,
        name: &'a T,
        type_definition: &'a TypeDefinition<T>,
        implements_interfaces: &ImplementsInterfaces<T>,
        interface_named_type: &'a NamedType<T>,
    ) -> Vec<Error<'a, T>> {
        let interface_definition = match self
            .0
            .type_definitions_by_name(interface_named_type.0.as_ref())
            .next()
        {
            Some(&TypeDefinition::InterfaceTypeDefinition(ref definition)) => definition,
            Some(_) => {
                return vec![Error::ImplementsNonInterfaceType {
                    name,
                    interface: interface_named_type.0.as_ref(),
                    span: interface_named_type.span(),
                }]
            }
            None => return vec![],
        };

        let mut errors = vec![];

        errors.extend(self.check_inherited_implementations(
            interface_definition,
            interface_named_type,
            name,
            implements_interfaces,
        ));

        errors.extend(self.check_valid_implementation(
            interface_named_type,
            interface_definition,
            name,
            type_definition,
        ));

        errors
    }

    pub fn check_type(
        &self,
        name: &'a T,
        type_definition: &'a TypeDefinition<T>,
        implements_interfaces: &'a ImplementsInterfaces<T>,
    ) -> Vec<Error<'a, T>> {
        let mut errors = vec![];
        let mut visited = HashMap::<&T, &NamedType<T>>::new();

        for interface in implements_interfaces.named_types() {
            if interface.0.as_ref() == name {
                errors.push(Error::SelfReferentialInterface {
                    name: interface.0.as_ref(),
                    span: interface.span(),
                });

                continue;
            }

            match visited.get(&interface.0.as_ref()) {
                Some(exists) => {
                    errors.push(Error::DuplicateImplementsInterface {
                        name: interface.0.as_ref(),
                        first: exists.span(),
                        second: interface.span(),
                    });
                    continue;
                }
                None => {}
            }

            visited.insert(interface.0.as_ref(), interface);

            errors.extend(self.check_interface(
                name,
                type_definition,
                implements_interfaces,
                interface,
            ));
        }

        errors
    }
}

impl<'a, T> Visit<'a, T> for ImplementsInterface<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_type_definition(
        &self,
        node: &'a Arc<TypeDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let ty = match node.name().ok() {
            Some(ty) => ty,
            None => return,
        };

        let implements_interfaces = match node.implements_interfaces() {
            Some(interfaces) => interfaces,
            None => return,
        };

        accumulator.extend(self.check_type(ty.as_ref(), node.as_ref(), implements_interfaces))
    }
}

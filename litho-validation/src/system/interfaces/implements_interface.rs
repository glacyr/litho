use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ImplementsInterface<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> ImplementsInterface<'a, T>
where
    T: Eq + Hash + ToString,
{
    fn check_inherited_implementations(
        &self,
        interface_named_type: &'a NamedType<T>,
        name: &'a T,
        implements_interfaces: &ImplementsInterfaces<T>,
    ) -> Option<Diagnostic<Span>> {
        for inherited in self
            .0
            .implemented_interfaces(interface_named_type.0.as_ref())
        {
            if !implements_interfaces.implements_interface(inherited.0.as_ref()) {
                return Some(Diagnostic::missing_inherited_interface(
                    name.to_string(),
                    interface_named_type.0.as_ref().to_string(),
                    inherited.0.as_ref().to_string(),
                    interface_named_type.span(),
                ));
            }
        }

        None
    }

    fn check_valid_implementation(
        &self,
        interface_name: &'a NamedType<T>,
        concrete_name: &'a T,
    ) -> Vec<Diagnostic<Span>> {
        let interface_fields = self.0.field_definitions(interface_name.0.as_ref());

        let mut errors = vec![];

        for field in interface_fields {
            let implemented_field = self
                .0
                .field_definitions_by_name(concrete_name, field.name.as_ref())
                .next();

            let implemented_field = match implemented_field {
                Some(field) => field,
                None => {
                    errors.push(Diagnostic::missing_interface_field(
                        concrete_name.to_string(),
                        interface_name.0.as_ref().to_string(),
                        field.name.as_ref().to_string(),
                        interface_name.span(),
                    ));

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
                                errors.push(Diagnostic::invalid_interface_field_argument_type(
                                    concrete_name.to_string(),
                                    interface_name.0.as_ref().to_string(),
                                    field.name.as_ref().to_string(),
                                    argument.name.as_ref().to_string(),
                                    expected.to_string(),
                                    ty.to_string(),
                                    ty.span(),
                                ));
                            }
                            _ => {}
                        }
                    }
                    None => errors.push(Diagnostic::missing_interface_field_argument(
                        concrete_name.to_string(),
                        interface_name.0.as_ref().to_string(),
                        field.name.as_ref().to_string(),
                        argument.name.as_ref().to_string(),
                        implemented_field.name.span(),
                    )),
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
                        errors.push(
                            Diagnostic::unexpected_non_null_extra_interface_field_argument(
                                concrete_name.to_string(),
                                interface_name.0.as_ref().to_string(),
                                field.name.as_ref().to_string(),
                                implemented_argument.name.as_ref().to_string(),
                                ty.to_string(),
                                ty.span(),
                            ),
                        );
                    }
                    _ => {}
                }
            }

            match field.ty.ok().zip(implemented_field.ty.ok()) {
                Some((field_type, implemented_field_type))
                    if !self
                        .is_valid_implementation_field_type(implemented_field_type, field_type) =>
                {
                    errors.push(Diagnostic::non_covariant_interface_field(
                        concrete_name.to_string(),
                        interface_name.0.as_ref().to_string(),
                        field.name.as_ref().to_string(),
                        field_type.to_string(),
                        implemented_field_type.to_string(),
                        implemented_field_type.span(),
                    ));
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
        implements_interfaces: &ImplementsInterfaces<T>,
        interface_named_type: &'a NamedType<T>,
    ) -> Vec<Diagnostic<Span>> {
        match self
            .0
            .type_definitions_by_name(interface_named_type.0.as_ref())
            .next()
            .map(AsRef::as_ref)
        {
            Some(&TypeDefinition::InterfaceTypeDefinition(_)) | None => {}
            Some(_) => {
                return vec![Diagnostic::implements_non_interface_type(
                    name.to_string(),
                    interface_named_type.0.as_ref().to_string(),
                    interface_named_type.span(),
                )];
            }
        };

        let mut errors = vec![];

        errors.extend(self.check_inherited_implementations(
            interface_named_type,
            name,
            implements_interfaces,
        ));

        errors.extend(self.check_valid_implementation(interface_named_type, name));

        errors
    }

    pub fn check_type(
        &self,
        name: &'a T,
        implements_interfaces: &'a ImplementsInterfaces<T>,
    ) -> Vec<Diagnostic<Span>> {
        let mut errors = vec![];

        for interface in implements_interfaces.named_types() {
            if interface.0.as_ref() == name {
                errors.push(Diagnostic::self_referential_interface(
                    interface.0.as_ref().to_string(),
                    interface.span(),
                ));

                continue;
            }

            match self
                .0
                .implemented_interfaces_by_name(name, interface.0.as_ref())
                .next()
            {
                Some(exists) if !Arc::ptr_eq(exists, interface) => {
                    errors.push(Diagnostic::duplicate_implements_interface(
                        name.to_string(),
                        interface.0.as_ref().to_string(),
                        exists.span(),
                        interface.span(),
                    ));
                    continue;
                }
                Some(_) | None => {}
            }

            errors.extend(self.check_interface(name, implements_interfaces, interface));
        }

        errors
    }
}

impl<'a, T> Visit<'a, T> for ImplementsInterface<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_type_definition(
        &self,
        node: &'a Arc<TypeDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(ty) = node.name().ok() else { return };

        let Some(implements_interfaces) = node.implements_interfaces() else {
            return;
        };

        accumulator.extend(self.check_type(ty.as_ref(), implements_interfaces))
    }

    fn visit_type_extension(
        &self,
        node: &'a Arc<TypeExtension<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(ty) = node.name() else { return };

        let Some(implements_interfaces) = node.implements_interfaces() else {
            return;
        };

        accumulator.extend(self.check_type(ty, implements_interfaces))
    }
}

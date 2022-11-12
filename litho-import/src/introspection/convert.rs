use std::iter::once;
use std::sync::Arc;

use litho_language::ast::*;

use crate::introspection;

pub trait Retrospect<T> {
    fn retrospect(self) -> Option<T>;
}

impl<T> Into<Document<T>> for introspection::Schema
where
    for<'a> T: From<&'a str>,
{
    fn into(self) -> Document<T> {
        Document {
            definitions: once(Definition::TypeSystemDefinitionOrExtension(
                TypeSystemDefinitionOrExtension::TypeSystemDefinition(
                    TypeSystemDefinition::SchemaDefinition(SchemaDefinition {
                        schema: Name::new(T::from("schema")),
                        type_definitions: Recoverable::Present(RootOperationTypeDefinitions {
                            braces: (
                                Punctuator::new(T::from("{")),
                                Recoverable::Present(Punctuator::new(T::from("}"))),
                            ),
                            definitions: Recoverable::Present(
                                vec![
                                    (
                                        OperationType::Query(Name::new(T::from("query"))),
                                        self.query_type.name.as_ref(),
                                    ),
                                    (
                                        OperationType::Mutation(Name::new(T::from("mutation"))),
                                        self.mutation_type.as_ref().and_then(|ty| ty.name.as_ref()),
                                    ),
                                    (
                                        OperationType::Subscription(Name::new(T::from(
                                            "subscription",
                                        ))),
                                        self.subscription_type
                                            .as_ref()
                                            .and_then(|ty| ty.name.as_ref()),
                                    ),
                                ]
                                .into_iter()
                                .flat_map(|(operation_type, name)| {
                                    Some(RootOperationTypeDefinition {
                                        operation_type,
                                        colon: Recoverable::Present(Punctuator::new(T::from(":"))),
                                        named_type: Recoverable::Present(NamedType(Name::new(
                                            T::from(name?),
                                        ))),
                                    })
                                })
                                .collect(),
                            ),
                        }),
                        description: None,
                        directives: None,
                    }),
                ),
            ))
            .chain(self.types.into_iter().flat_map(|ty: introspection::Type| {
                ty.retrospect()
                    .map(Arc::new)
                    .map(TypeSystemDefinition::TypeDefinition)
                    .map(TypeSystemDefinitionOrExtension::TypeSystemDefinition)
                    .map(Definition::TypeSystemDefinitionOrExtension)
            }))
            .map(Arc::new)
            .collect(),
        }
    }
}

impl<T> Retrospect<Description<T>> for Option<String>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<Description<T>> {
        self.and_then(|description| match description.is_empty() {
            false => Some(Description(StringValue::block(description))),
            true => None,
        })
    }
}

impl<T> Retrospect<TypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<TypeDefinition<T>> {
        match self.kind {
            introspection::TypeKind::Enum => {
                Some(TypeDefinition::EnumTypeDefinition(self.retrospect()?))
            }
            introspection::TypeKind::Object => {
                Some(TypeDefinition::ObjectTypeDefinition(self.retrospect()?))
            }
            introspection::TypeKind::InputObject => Some(
                TypeDefinition::InputObjectTypeDefinition(self.retrospect()?),
            ),
            introspection::TypeKind::Interface => {
                Some(TypeDefinition::InterfaceTypeDefinition(self.retrospect()?))
            }
            introspection::TypeKind::Scalar => {
                Some(TypeDefinition::ScalarTypeDefinition(self.retrospect()?))
            }
            introspection::TypeKind::Union => {
                Some(TypeDefinition::UnionTypeDefinition(self.retrospect()?))
            }
            _ => None,
        }
    }
}

impl<T> Retrospect<EnumTypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<EnumTypeDefinition<T>> {
        Some(EnumTypeDefinition {
            enum_kw: Name::new(T::from("enum")),
            name: Recoverable::Present(Name::new(T::from(&self.name?))),
            values_definition: self.enum_values.and_then(Retrospect::retrospect),
            description: self.description.retrospect(),
            directives: None,
        })
    }
}

impl<T> Retrospect<EnumValuesDefinition<T>> for Vec<introspection::EnumValue>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<EnumValuesDefinition<T>> {
        if self.is_empty() {
            return None;
        }

        Some(EnumValuesDefinition {
            braces: (
                Punctuator::new(T::from("{")),
                Recoverable::Present(Punctuator::new(T::from("}"))),
            ),
            definitions: self
                .into_iter()
                .flat_map(Retrospect::retrospect)
                .map(Arc::new)
                .collect(),
        })
    }
}

impl<T> Retrospect<EnumValueDefinition<T>> for introspection::EnumValue
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<EnumValueDefinition<T>> {
        Some(EnumValueDefinition {
            enum_value: EnumValue(Name::new(T::from(&self.name))),
            description: self.description.retrospect(),
            directives: None,
        })
    }
}

impl<T> Retrospect<InputObjectTypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<InputObjectTypeDefinition<T>> {
        Some(InputObjectTypeDefinition {
            input: Name::new(T::from("input")),
            name: Recoverable::Present(Name::new(T::from(&self.name?))),
            description: self.description.retrospect(),
            directives: None,
            fields_definition: self.input_fields.and_then(Retrospect::retrospect),
        })
    }
}

impl<T> Retrospect<InputFieldsDefinition<T>> for Vec<introspection::InputValue>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<InputFieldsDefinition<T>> {
        if self.is_empty() {
            return None;
        }

        Some(InputFieldsDefinition {
            braces: (
                Punctuator::new(T::from("{")),
                Recoverable::Present(Punctuator::new(T::from("}"))),
            ),
            definitions: self
                .into_iter()
                .flat_map(|field| field.retrospect())
                .map(Arc::new)
                .collect(),
        })
    }
}

impl<T> Retrospect<InterfaceTypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<InterfaceTypeDefinition<T>> {
        Some(InterfaceTypeDefinition {
            interface: Name::new(T::from("interface")),
            name: Recoverable::Present(Name::new(T::from(&self.name?))),
            description: self.description.retrospect(),
            directives: None,
            fields_definition: self.fields.and_then(Retrospect::retrospect),
            implements_interfaces: self.interfaces.and_then(Retrospect::retrospect),
        })
    }
}

impl<T> Retrospect<ObjectTypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<ObjectTypeDefinition<T>> {
        Some(ObjectTypeDefinition {
            ty: Name::new(T::from("type")),
            name: Recoverable::Present(Name::new(T::from(&self.name?))),
            description: self.description.retrospect(),
            directives: None,
            fields_definition: self.fields.and_then(Retrospect::retrospect),
            implements_interfaces: self.interfaces.and_then(Retrospect::retrospect),
        })
    }
}

impl<T> Retrospect<FieldsDefinition<T>> for Vec<introspection::Field>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<FieldsDefinition<T>> {
        Some(FieldsDefinition {
            braces: (
                Punctuator::new(T::from("{")),
                Recoverable::Present(Punctuator::new(T::from("}"))),
            ),
            definitions: self
                .into_iter()
                .flat_map(|field| field.retrospect())
                .map(Arc::new)
                .collect(),
        })
    }
}

impl<T> Retrospect<FieldDefinition<T>> for introspection::Field
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<FieldDefinition<T>> {
        Some(FieldDefinition {
            name: Name::new(T::from(&self.name)),
            colon: Recoverable::Present(Punctuator::new(T::from(":"))),
            arguments_definition: self.args.retrospect().map(Arc::new),
            description: self.description.retrospect(),
            directives: None,
            ty: Recoverable::Present(self.ty.retrospect().map(Arc::new)?),
        })
    }
}

impl<T> Retrospect<ArgumentsDefinition<T>> for Vec<super::InputValue>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<ArgumentsDefinition<T>> {
        if self.is_empty() {
            return None;
        }

        Some(ArgumentsDefinition {
            parens: (
                Punctuator::new(T::from("(")),
                Recoverable::Present(Punctuator::new(T::from(")"))),
            ),
            definitions: self
                .into_iter()
                .flat_map(Retrospect::retrospect)
                .map(Arc::new)
                .collect(),
        })
    }
}

impl<T> Retrospect<InputValueDefinition<T>> for super::InputValue
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<InputValueDefinition<T>> {
        Some(InputValueDefinition {
            name: Name::new(T::from(&self.name)),
            colon: Recoverable::Present(Punctuator::new(T::from(":"))),
            ty: Recoverable::Present(self.ty.retrospect().map(Arc::new)?),
            description: self.description.retrospect(),
            default_value: None,
            directives: None,
        })
    }
}

impl<T> Retrospect<Type<T>> for super::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<Type<T>> {
        match self.kind {
            super::TypeKind::Enum
            | super::TypeKind::InputObject
            | super::TypeKind::Interface
            | super::TypeKind::Object
            | super::TypeKind::Scalar
            | super::TypeKind::Union => {
                Some(Type::Named(NamedType(Name::new(T::from(&self.name?)))))
            }
            super::TypeKind::NonNull => Some(Type::NonNull(NonNullType {
                ty: self.of_type.and_then(|ty| ty.retrospect()).map(Arc::new)?,
                bang: Punctuator::new(T::from("!")),
            })),
            super::TypeKind::List => Some(Type::List(ListType {
                brackets: (
                    Punctuator::new(T::from("[")),
                    Recoverable::Present(Punctuator::new(T::from("]"))),
                ),
                ty: Recoverable::Present(
                    self.of_type.and_then(|ty| ty.retrospect()).map(Arc::new)?,
                ),
            })),
        }
    }
}

impl<T> Retrospect<ScalarTypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<ScalarTypeDefinition<T>> {
        Some(ScalarTypeDefinition {
            scalar: Name::new(T::from("scalar")),
            name: Recoverable::Present(Name::new(T::from(&self.name?))),
            description: self.description.retrospect(),
            directives: None,
        })
    }
}

impl<T> Retrospect<UnionTypeDefinition<T>> for introspection::Type
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<UnionTypeDefinition<T>> {
        Some(UnionTypeDefinition {
            union_kw: Name::new(T::from("union")),
            name: Recoverable::Present(Name::new(T::from(&self.name?))),
            member_types: self.possible_types.and_then(Retrospect::retrospect),
            description: self.description.retrospect(),
            directives: None,
        })
    }
}

impl<T> Retrospect<UnionMemberTypes<T>> for Vec<introspection::Type>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<UnionMemberTypes<T>> {
        Some(UnionMemberTypes {
            eq: Punctuator::new(T::from("=")),
            first: Recoverable::Present(Arc::new(NamedType(Name::new(T::from(
                self.first().and_then(|ty| ty.name.as_ref())?,
            ))))),
            pipe: None,
            types: self
                .into_iter()
                .skip(1)
                .flat_map(|ty| {
                    ty.name.map(|name| {
                        (
                            Punctuator::new(T::from("|")),
                            Recoverable::Present(Arc::new(NamedType(Name::new(T::from(&name))))),
                        )
                    })
                })
                .collect(),
        })
    }
}

impl<T> Retrospect<ImplementsInterfaces<T>> for Vec<introspection::Type>
where
    for<'a> T: From<&'a str>,
{
    fn retrospect(self) -> Option<ImplementsInterfaces<T>> {
        Some(ImplementsInterfaces {
            implements: Name::new(T::from("implements")),
            first: Recoverable::Present(Arc::new(NamedType(Name::new(T::from(
                self.first().and_then(|ty| ty.name.as_ref())?,
            ))))),
            ampersand: None,
            types: self
                .into_iter()
                .skip(1)
                .flat_map(|ty| {
                    ty.name.map(|name| {
                        (
                            Punctuator::new(T::from("&")),
                            Recoverable::Present(Arc::new(NamedType(Name::new(T::from(&name))))),
                        )
                    })
                })
                .collect(),
        })
    }
}

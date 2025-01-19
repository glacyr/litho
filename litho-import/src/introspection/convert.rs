use std::iter::once;

use litho_language::ast::*;

use crate::introspection;

pub trait Retrospect<'a, T, U>
where
    T: ContextValue<'a>,
{
    fn retrospect<C>(self, context: &C) -> Option<U>
    where
        C: Context<'a, T>;
}

impl<'a, T> Retrospect<'a, T, Document<'a, T>> for introspection::Schema
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<Document<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(Document {
            definitions: context.list_from_iter(
                once(Definition::TypeSystemDefinitionOrExtension(
                    TypeSystemDefinitionOrExtension::TypeSystemDefinition(
                        TypeSystemDefinition::SchemaDefinition(SchemaDefinition {
                            schema: Name::new("schema"),
                            type_definitions: Recoverable::Present(RootOperationTypeDefinitions {
                                braces: (
                                    Punctuator::new("{"),
                                    Recoverable::Present(Punctuator::new("}")),
                                ),
                                definitions: Recoverable::Present(
                                    context.list_from_iter(
                                        vec![
                                            (
                                                OperationType::Query(Name::new("query")),
                                                self.query_type.name.as_ref(),
                                            ),
                                            (
                                                OperationType::Mutation(Name::new("mutation")),
                                                self.mutation_type
                                                    .as_ref()
                                                    .and_then(|ty| ty.name.as_ref()),
                                            ),
                                            (
                                                OperationType::Subscription(Name::new(
                                                    "subscription",
                                                )),
                                                self.subscription_type
                                                    .as_ref()
                                                    .and_then(|ty| ty.name.as_ref()),
                                            ),
                                        ]
                                        .into_iter()
                                        .flat_map(
                                            |(operation_type, name)| {
                                                Some(RootOperationTypeDefinition {
                                                    operation_type,
                                                    colon: Recoverable::Present(Punctuator::new(
                                                        ":",
                                                    )),
                                                    named_type: Recoverable::Present(NamedType(
                                                        Name::new(name?),
                                                    )),
                                                })
                                            },
                                        ),
                                    ),
                                ),
                            }),
                            description: None,
                            directives: None,
                        }),
                    ),
                ))
                .chain(self.types.into_iter().flat_map(|ty: introspection::Type| {
                    ty.retrospect(context)
                        .map(|ty| context.shared(ty))
                        .map(TypeSystemDefinition::TypeDefinition)
                        .map(TypeSystemDefinitionOrExtension::TypeSystemDefinition)
                        .map(Definition::TypeSystemDefinitionOrExtension)
                }))
                .map(|definition| context.shared(definition)),
            ),
        })
    }
}

impl<'a, T> Retrospect<'a, T, Description<'a, T>> for Option<String>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, _context: &C) -> Option<Description<'a, T>>
    where
        C: Context<'a, T>,
    {
        self.and_then(|description| match description.is_empty() {
            false => Some(Description(StringValue::block(description))),
            true => None,
        })
    }
}

impl<'a, T> Retrospect<'a, T, TypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<TypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        match self.name.as_ref().map(String::as_str)? {
            "Int" | "Float" | "String" | "Boolean" | "ID" => return None,
            name if name.starts_with("__") => return None,
            _ => {}
        }

        match self.kind {
            introspection::TypeKind::Enum => Some(TypeDefinition::EnumTypeDefinition(
                self.retrospect(context)?,
            )),
            introspection::TypeKind::Object => Some(TypeDefinition::ObjectTypeDefinition(
                self.retrospect(context)?,
            )),
            introspection::TypeKind::InputObject => Some(
                TypeDefinition::InputObjectTypeDefinition(self.retrospect(context)?),
            ),
            introspection::TypeKind::Interface => Some(TypeDefinition::InterfaceTypeDefinition(
                self.retrospect(context)?,
            )),
            introspection::TypeKind::Scalar => Some(TypeDefinition::ScalarTypeDefinition(
                self.retrospect(context)?,
            )),
            introspection::TypeKind::Union => Some(TypeDefinition::UnionTypeDefinition(
                self.retrospect(context)?,
            )),
            _ => None,
        }
    }
}

impl<'a, T> Retrospect<'a, T, EnumTypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<EnumTypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(EnumTypeDefinition {
            enum_kw: Name::new("enum"),
            name: Recoverable::Present(Name::new(&self.name?)),
            values_definition: self.enum_values.and_then(|value| value.retrospect(context)),
            description: self.description.retrospect(context),
            directives: None,
        })
    }
}

impl<'a, T> Retrospect<'a, T, EnumValuesDefinition<'a, T>> for Vec<introspection::EnumValue>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<EnumValuesDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        if self.is_empty() {
            return None;
        }

        Some(EnumValuesDefinition {
            braces: (
                Punctuator::new("{"),
                Recoverable::Present(Punctuator::new("}")),
            ),
            definitions: context.list_from_iter(
                self.into_iter()
                    .flat_map(|definition| definition.retrospect(context))
                    .map(|definition| context.shared(definition)),
            ),
        })
    }
}

impl<'a, T> Retrospect<'a, T, EnumValueDefinition<'a, T>> for introspection::EnumValue
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<EnumValueDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(EnumValueDefinition {
            enum_value: EnumValue(Name::new(&self.name)),
            description: self.description.retrospect(context),
            directives: None,
        })
    }
}

impl<'a, T> Retrospect<'a, T, InputObjectTypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<InputObjectTypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(InputObjectTypeDefinition {
            input: Name::new("input"),
            name: Recoverable::Present(Name::new(&self.name?)),
            description: self.description.retrospect(context),
            directives: None,
            fields_definition: self
                .input_fields
                .and_then(|definition| definition.retrospect(context)),
        })
    }
}

impl<'a, T> Retrospect<'a, T, InputFieldsDefinition<'a, T>> for Vec<introspection::InputValue>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<InputFieldsDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        if self.is_empty() {
            return None;
        }

        Some(InputFieldsDefinition {
            braces: (
                Punctuator::new("{"),
                Recoverable::Present(Punctuator::new("}")),
            ),
            definitions: context.list_from_iter(
                self.into_iter()
                    .flat_map(|field| field.retrospect(context))
                    .map(|definition| context.shared(definition)),
            ),
        })
    }
}

impl<'a, T> Retrospect<'a, T, InterfaceTypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<InterfaceTypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(InterfaceTypeDefinition {
            interface: Name::new("interface"),
            name: Recoverable::Present(Name::new(&self.name?)),
            description: self.description.retrospect(context),
            directives: None,
            fields_definition: self
                .fields
                .and_then(|definition| definition.retrospect(context)),
            implements_interfaces: self
                .interfaces
                .and_then(|definition| definition.retrospect(context)),
        })
    }
}

impl<'a, T> Retrospect<'a, T, ObjectTypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<ObjectTypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(ObjectTypeDefinition {
            ty: Name::new("type"),
            name: Recoverable::Present(Name::new(&self.name?)),
            description: self.description.retrospect(context),
            directives: None,
            fields_definition: self
                .fields
                .and_then(|definition| definition.retrospect(context)),
            implements_interfaces: self
                .interfaces
                .and_then(|definition| definition.retrospect(context)),
        })
    }
}

impl<'a, T> Retrospect<'a, T, FieldsDefinition<'a, T>> for Vec<introspection::Field>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<FieldsDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(FieldsDefinition {
            braces: (
                Punctuator::new("{"),
                Recoverable::Present(Punctuator::new("}")),
            ),
            definitions: context.list_from_iter(
                self.into_iter()
                    .flat_map(|field| field.retrospect(context))
                    .map(|definition| context.shared(definition)),
            ),
        })
    }
}

impl<'a, T> Retrospect<'a, T, FieldDefinition<'a, T>> for introspection::Field
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<FieldDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(FieldDefinition {
            name: Name::new(&self.name),
            colon: Recoverable::Present(Punctuator::new(":")),
            arguments_definition: self
                .args
                .retrospect(context)
                .map(|definition| context.shared(definition)),
            description: self.description.retrospect(context),
            directives: None,
            ty: Recoverable::Present(
                self.ty
                    .retrospect(context)
                    .map(|definition| context.shared(definition))?,
            ),
        })
    }
}

impl<'a, T> Retrospect<'a, T, ArgumentsDefinition<'a, T>> for Vec<super::InputValue>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<ArgumentsDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        if self.is_empty() {
            return None;
        }

        Some(ArgumentsDefinition {
            parens: (
                Punctuator::new("("),
                Recoverable::Present(Punctuator::new(")")),
            ),
            definitions: context.list_from_iter(
                self.into_iter()
                    .flat_map(|definition| definition.retrospect(context))
                    .map(|definition| context.shared(definition)),
            ),
        })
    }
}

impl<'a, T> Retrospect<'a, T, InputValueDefinition<'a, T>> for super::InputValue
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<InputValueDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(InputValueDefinition {
            name: Name::new(&self.name),
            colon: Recoverable::Present(Punctuator::new(":")),
            ty: Recoverable::Present(self.ty.retrospect(context).map(|ty| context.shared(ty))?),
            description: self.description.retrospect(context),
            default_value: None,
            directives: None,
        })
    }
}

impl<'a, T> Retrospect<'a, T, Type<'a, T>> for super::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<Type<'a, T>>
    where
        C: Context<'a, T>,
    {
        match self.kind {
            super::TypeKind::Enum
            | super::TypeKind::InputObject
            | super::TypeKind::Interface
            | super::TypeKind::Object
            | super::TypeKind::Scalar
            | super::TypeKind::Union => Some(Type::Named(NamedType(Name::new(&self.name?)))),
            super::TypeKind::NonNull => Some(Type::NonNull(NonNullType {
                ty: self
                    .of_type
                    .and_then(|ty| ty.retrospect(context))
                    .map(|ty| context.shared(ty))?,
                bang: Punctuator::new("!"),
            })),
            super::TypeKind::List => Some(Type::List(ListType {
                brackets: (
                    Punctuator::new("["),
                    Recoverable::Present(Punctuator::new("]")),
                ),
                ty: Recoverable::Present(
                    self.of_type
                        .and_then(|ty| ty.retrospect(context))
                        .map(|ty| context.shared(ty))?,
                ),
            })),
        }
    }
}

impl<'a, T> Retrospect<'a, T, ScalarTypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<ScalarTypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(ScalarTypeDefinition {
            scalar: Name::new("scalar"),
            name: Recoverable::Present(Name::new(&self.name?)),
            description: self.description.retrospect(context),
            directives: None,
        })
    }
}

impl<'a, T> Retrospect<'a, T, UnionTypeDefinition<'a, T>> for introspection::Type
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<UnionTypeDefinition<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(UnionTypeDefinition {
            union_kw: Name::new("union"),
            name: Recoverable::Present(Name::new(&self.name?)),
            member_types: self.possible_types.and_then(|ty| ty.retrospect(context)),
            description: self.description.retrospect(context),
            directives: None,
        })
    }
}

impl<'a, T> Retrospect<'a, T, UnionMemberTypes<'a, T>> for Vec<introspection::Type>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<UnionMemberTypes<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(UnionMemberTypes {
            eq: Punctuator::new("="),
            first: Recoverable::Present(context.shared(NamedType(Name::new(
                self.first().and_then(|ty| ty.name.as_ref())?,
            )))),
            pipe: None,
            types: context.list_from_iter(self.into_iter().skip(1).flat_map(|ty| {
                ty.name.map(|name| {
                    (
                        Punctuator::new("|"),
                        Recoverable::Present(context.shared(NamedType(Name::new(&name)))),
                    )
                })
            })),
        })
    }
}

impl<'a, T> Retrospect<'a, T, ImplementsInterfaces<'a, T>> for Vec<introspection::Type>
where
    T: ContextValue<'a>,
    for<'b> T: From<&'b str>,
{
    fn retrospect<C>(self, context: &C) -> Option<ImplementsInterfaces<'a, T>>
    where
        C: Context<'a, T>,
    {
        Some(ImplementsInterfaces {
            implements: Name::new("implements"),
            first: Recoverable::Present(context.shared(NamedType(Name::new(
                self.first().and_then(|ty| ty.name.as_ref())?,
            )))),
            ampersand: None,
            types: context.list_from_iter(self.into_iter().skip(1).flat_map(|ty| {
                ty.name.map(|name| {
                    (
                        Punctuator::new("&"),
                        Recoverable::Present(context.shared(NamedType(Name::new(&name)))),
                    )
                })
            })),
        })
    }
}

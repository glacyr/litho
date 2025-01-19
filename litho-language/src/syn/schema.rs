use litho_diagnostics::Diagnostic;
use wrom::{alt, delimited, opt, Input, RecoverableParser};
use wrom_derive::wrom;

use crate::ast::*;
use crate::lex::{Token, TokenKind};

use super::combinators::{keyword, name, name_unless_on, punctuator, string_value};
use super::executable::{default_value, directives, enum_value, named_type, operation_type, ty};
use super::recovery::RecoveryPoint;
use super::{many_ext, Error, RecoverableParserExt, RECURSION_LIMIT};

#[wrom]
pub fn type_system_document<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDocument<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    many_ext(type_system_definition()).map(|definitions| TypeSystemDocument { definitions })
}

#[wrom]
pub fn type_system_definition<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        description().flat_map(|description| {
            opt(type_system_definition_with_description(Some(
                description.clone(),
            )))
            .map(move |opt| opt.unwrap_or_else(|| TypeSystemDefinition::Error(description.clone())))
        }),
        type_system_definition_with_description(None),
    ))
}

#[wrom]
pub fn type_system_definition_with_description<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, TypeSystemDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        schema_definition(description.clone()).map(TypeSystemDefinition::SchemaDefinition),
        type_definition(description.clone())
            .into_shared()
            .map(TypeSystemDefinition::TypeDefinition),
        directive_definition(description)
            .into_shared()
            .map(TypeSystemDefinition::DirectiveDefinition),
    ))
}

#[wrom]
pub fn type_system_extension_document<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemExtensionDocument<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    many_ext(type_system_definition_or_extension())
        .map(|definitions| TypeSystemExtensionDocument { definitions })
}

#[wrom]
pub fn type_system_definition_or_extension<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDefinitionOrExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        type_system_definition().map(TypeSystemDefinitionOrExtension::TypeSystemDefinition),
        type_system_extension().map(TypeSystemDefinitionOrExtension::TypeSystemExtension),
    ))
}

#[wrom]
pub fn type_system_extension<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    keyword(TokenKind::KeywordExtend).flat_map(type_system_extension_with_extend)
}

#[wrom]
pub fn type_system_extension_with_extend<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, TypeSystemExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    opt(alt((
        schema_extension(extend.clone()).map(TypeSystemExtension::SchemaExtension),
        type_extension(extend.clone())
            .into_shared()
            .map(TypeSystemExtension::TypeExtension),
    )))
    .map(move |opt| opt.unwrap_or(TypeSystemExtension::Error(extend.clone())))
}

#[wrom]
pub fn description<'a, T, I>() -> impl RecoverableParser<I, Description<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    string_value().map(Description)
}

#[wrom]
pub fn schema_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, SchemaDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordSchema),
        opt(directives()),
        root_operation_type_definitions().recover(Missing::unary(
            Diagnostic::missing_root_operation_type_definitions,
        )),
    )
        .map(
            move |(schema, directives, type_definitions)| SchemaDefinition {
                description: description.clone(),
                schema,
                directives,
                type_definitions,
            },
        )
}

#[wrom]
pub fn root_operation_type_definitions<'a, T, I>(
) -> impl RecoverableParser<I, RootOperationTypeDefinitions<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many_ext(root_operation_type_definition()).recover(Missing::unary(
            Diagnostic::missing_root_operation_type_definitions,
        )),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_root_operation_type_definitions_closing_brace),
    )
    .map(|(left, definitions, right)| RootOperationTypeDefinitions {
        braces: (left, right),
        definitions,
    })
}

#[wrom]
pub fn root_operation_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, RootOperationTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        operation_type(),
        punctuator(TokenKind::Colon).recover(Missing::unary(
            Diagnostic::missing_root_operation_type_definition_colon,
        )),
        named_type().recover(Missing::unary(
            Diagnostic::missing_root_operation_type_definition_named_type,
        )),
    )
        .map(
            |(operation_type, colon, named_type)| RootOperationTypeDefinition {
                operation_type,
                colon,
                named_type,
            },
        )
}

#[wrom]
pub fn schema_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, SchemaExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordSchema),
        opt(directives()),
        opt(root_operation_type_definitions()),
    )
        .map(
            move |(schema, directives, type_definitions)| SchemaExtension {
                extend_schema: (extend.clone(), schema),
                directives,
                type_definitions,
            },
        )
}

#[wrom]
pub fn type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, TypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        scalar_type_definition(description.clone()).map(TypeDefinition::ScalarTypeDefinition),
        object_type_definition(description.clone()).map(TypeDefinition::ObjectTypeDefinition),
        interface_type_definition(description.clone()).map(TypeDefinition::InterfaceTypeDefinition),
        union_type_definition(description.clone()).map(TypeDefinition::UnionTypeDefinition),
        enum_type_definition(description.clone()).map(TypeDefinition::EnumTypeDefinition),
        input_object_type_definition(description.clone())
            .map(TypeDefinition::InputObjectTypeDefinition),
    ))
}

#[wrom]
pub fn type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, TypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        scalar_type_extension(extend.clone()).map(TypeExtension::ScalarTypeExtension),
        object_type_extension(extend.clone()).map(TypeExtension::ObjectTypeExtension),
        interface_type_extension(extend.clone()).map(TypeExtension::InterfaceTypeExtension),
        union_type_extension(extend.clone()).map(TypeExtension::UnionTypeExtension),
        enum_type_extension(extend.clone()).map(TypeExtension::EnumTypeExtension),
        input_object_type_extension(extend.clone()).map(TypeExtension::InputObjectTypeExtension),
    ))
}

#[wrom]
pub fn scalar_type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, ScalarTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    keyword(TokenKind::KeywordScalar)
        .and(name().recover(Missing::unary(
            Diagnostic::missing_scalar_type_definition_name,
        )))
        .and(opt(directives()))
        .map(|((scalar, name), directives)| ScalarTypeDefinition {
            description: description.clone(),
            scalar,
            name,
            directives,
        })
}

#[wrom]
pub fn scalar_type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, ScalarTypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordScalar),
        named_type().recover(Missing::unary(
            Diagnostic::missing_scalar_type_extension_name,
        )),
        directives().recover(Missing::unary(
            Diagnostic::missing_scalar_type_extension_directives,
        )),
    )
        .map(move |(scalar, name, directives)| ScalarTypeExtension {
            extend_scalar: (extend.clone(), scalar),
            name,
            directives,
        })
}

#[wrom]
pub fn object_type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, ObjectTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordType),
        name().recover(Missing::unary(
            Diagnostic::missing_object_type_definition_name,
        )),
        opt(implements_interfaces()),
        opt(directives()),
        opt(fields_definition()),
    )
        .map(
            |(ty, name, implements_interfaces, directives, fields_definition)| {
                ObjectTypeDefinition {
                    description: description.clone(),
                    ty,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom]
pub fn implements_interfaces<'a, T, I>(
) -> impl RecoverableParser<I, ImplementsInterfaces<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordImplements),
        opt(punctuator(TokenKind::Ampersand)),
        named_type().into_shared().recover(Missing::unary(
            Diagnostic::missing_first_implements_interface,
        )),
        many_ext(
            punctuator(TokenKind::Ampersand).and(named_type().into_shared().recover(
                Missing::unary(Diagnostic::missing_second_implements_interface),
            )),
        ),
    )
        .map(
            |(implements, ampersand, first, types)| ImplementsInterfaces {
                implements,
                ampersand,
                first,
                types,
            },
        )
}

#[wrom]
pub fn fields_definition<'a, T, I>() -> impl RecoverableParser<I, FieldsDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many_ext(field_definition().into_shared()),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_fields_definition_closing_brace),
    )
    .map(|(left, definitions, right)| FieldsDefinition {
        braces: (left, right),
        definitions,
    })
}

#[wrom]
pub fn field_definition<'a, T, I>() -> impl RecoverableParser<I, FieldDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    opt(description())
        .and_recognize((
            name(),
            opt(arguments_definition().into_shared()),
            punctuator(TokenKind::Colon)
                .recover(Missing::unary(Diagnostic::missing_field_definition_colon)),
            ty(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_field_definition_type)),
            opt(directives()),
        ))
        .map(
            |(description, (name, arguments_definition, colon, ty, directives))| FieldDefinition {
                description,
                name,
                arguments_definition,
                colon,
                ty,
                directives,
            },
        )
}

#[wrom]
pub fn arguments_definition<'a, T, I>(
) -> impl RecoverableParser<I, ArgumentsDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::ParenLeft),
        many_ext(input_value_definition().into_shared()),
        punctuator(TokenKind::ParenRight),
        Missing::binary(Diagnostic::missing_arguments_definition_closing_parenthesis),
    )
    .map(|(left, definitions, right)| ArgumentsDefinition {
        parens: (left, right),
        definitions,
    })
}

#[wrom]
pub fn input_value_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputValueDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    opt(description())
        .and_recognize((
            name(),
            punctuator(TokenKind::Colon).recover(Missing::unary(
                Diagnostic::missing_input_value_definition_colon,
            )),
            ty(RECURSION_LIMIT).recover(Missing::unary(
                Diagnostic::missing_input_value_definition_type,
            )),
            opt(default_value()),
            opt(directives()),
        ))
        .map(
            |(description, (name, colon, ty, default_value, directives))| InputValueDefinition {
                description,
                name,
                colon,
                ty,
                default_value,
                directives,
            },
        )
}

#[wrom]
pub fn object_type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, ObjectTypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordType),
        named_type().recover(Missing::unary(
            Diagnostic::missing_object_type_extension_name,
        )),
        opt(implements_interfaces()),
        opt(directives()),
        opt(fields_definition()),
    )
        .map(
            move |(ty, name, implements_interfaces, directives, fields_definition)| {
                ObjectTypeExtension {
                    extend_type: (extend.clone(), ty.into()),
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom]
pub fn interface_type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, InterfaceTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordInterface),
        name().recover(Missing::unary(
            Diagnostic::missing_interface_type_definition_name,
        )),
        opt(implements_interfaces()),
        opt(directives()),
        opt(fields_definition()),
    )
        .map(
            |(interface, name, implements_interfaces, directives, fields_definition)| {
                InterfaceTypeDefinition {
                    description: description.clone(),
                    interface,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom]
pub fn interface_type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, InterfaceTypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordInterface),
        named_type().recover(Missing::unary(
            Diagnostic::missing_interface_type_extension_name,
        )),
        opt(implements_interfaces()),
        opt(directives()),
        opt(fields_definition()),
    )
        .map(
            move |(interface, name, implements_interfaces, directives, fields_definition)| {
                InterfaceTypeExtension {
                    extend_interface: (extend.clone(), interface),
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom]
pub fn union_type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, UnionTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordUnion),
        name().recover(Missing::unary(
            Diagnostic::missing_union_type_definition_name,
        )),
        opt(directives()),
        opt(union_member_types()),
    )
        .map(
            |(union_kw, name, directives, member_types)| UnionTypeDefinition {
                description: description.clone(),
                union_kw,
                name,
                directives,
                member_types,
            },
        )
}

#[wrom]
pub fn union_member_types<'a, T, I>() -> impl RecoverableParser<I, UnionMemberTypes<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        punctuator(TokenKind::Eq),
        opt(punctuator(TokenKind::Pipe)),
        named_type()
            .into_shared()
            .recover(Missing::unary(Diagnostic::missing_first_union_member_type)),
        many_ext(
            punctuator(TokenKind::Pipe).and(
                named_type()
                    .into_shared()
                    .recover(Missing::unary(Diagnostic::missing_second_union_member_type)),
            ),
        ),
    )
        .map(|(eq, pipe, first, types)| UnionMemberTypes {
            eq,
            pipe,
            first,
            types,
        })
}

#[wrom]
pub fn union_type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, UnionTypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordUnion),
        named_type().recover(Missing::unary(
            Diagnostic::missing_union_type_extension_name,
        )),
        opt(directives()),
        opt(union_member_types()),
    )
        .map(
            move |(union_kw, name, directives, member_types)| UnionTypeExtension {
                extend_union: (extend.clone(), union_kw),
                name,
                directives,
                member_types,
            },
        )
}

#[wrom]
pub fn enum_type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, EnumTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordEnum),
        name().recover(Missing::unary(
            Diagnostic::missing_enum_type_definition_name,
        )),
        opt(directives()),
        opt(enum_values_definition()),
    )
        .map(
            |(enum_kw, name, directives, values_definition)| EnumTypeDefinition {
                description: description.clone(),
                enum_kw,
                name,
                directives,
                values_definition,
            },
        )
}

#[wrom]
pub fn enum_values_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumValuesDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many_ext(enum_value_definition().into_shared()),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_enum_values_closing_brace),
    )
    .map(|(left, definitions, right)| EnumValuesDefinition {
        braces: (left, right),
        definitions,
    })
}

#[wrom]
pub fn enum_value_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumValueDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    opt(description())
        .and_recognize(enum_value())
        .and(opt(directives()))
        .map(
            |((description, enum_value), directives)| EnumValueDefinition {
                description,
                enum_value,
                directives,
            },
        )
}

#[wrom]
pub fn enum_type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, EnumTypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordEnum),
        named_type().recover(Missing::unary(Diagnostic::missing_enum_type_extension_name)),
        opt(directives()),
        opt(enum_values_definition()),
    )
        .map(
            move |(enum_kw, name, directives, values_definition)| EnumTypeExtension {
                extend_enum: (extend.clone(), enum_kw),
                name,
                directives,
                values_definition,
            },
        )
}

#[wrom]
pub fn input_object_type_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, InputObjectTypeDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordInput),
        name().recover(Missing::unary(
            Diagnostic::missing_input_object_type_definition_name,
        )),
        opt(directives()),
        opt(input_fields_definition()),
    )
        .map(
            |(input, name, directives, fields_definition)| InputObjectTypeDefinition {
                description: description.clone(),
                input,
                name,
                directives,
                fields_definition,
            },
        )
}

#[wrom]
pub fn input_fields_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputFieldsDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many_ext(input_value_definition().into_shared()),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_input_fields_definition_closing_brace),
    )
    .map(|(left, definitions, right)| InputFieldsDefinition {
        braces: (left, right),
        definitions,
    })
}

#[wrom]
pub fn input_object_type_extension<'a, T, I>(
    extend: Name<'a, T>,
) -> impl RecoverableParser<I, InputObjectTypeExtension<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordInput),
        named_type().recover(Missing::unary(
            Diagnostic::missing_input_object_type_extension_name,
        )),
        opt(directives()),
        opt(input_fields_definition()),
    )
        .map(
            move |(input, name, directives, fields_definition)| InputObjectTypeExtension {
                extend_input: (extend.clone(), input),
                name,
                directives,
                fields_definition,
            },
        )
}

#[wrom]
pub fn directive_definition<'a, T, I>(
    description: Option<Description<'a, T>>,
) -> impl RecoverableParser<I, DirectiveDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordDirective),
        punctuator(TokenKind::At)
            .recover(Missing::unary(Diagnostic::missing_directive_definition_at)),
        name_unless_on().recover(Missing::unary(
            Diagnostic::missing_directive_definition_name,
        )),
        opt(arguments_definition().into_shared()),
        opt(keyword(TokenKind::KeywordRepeatable)),
        directive_locations().recover(Missing::unary(
            Diagnostic::missing_directive_definition_locations,
        )),
    )
        .map(
            |(directive, at, name, arguments_definition, repeatable, locations)| {
                DirectiveDefinition {
                    description: description.clone(),
                    directive,
                    at,
                    name,
                    arguments_definition,
                    repeatable,
                    locations,
                }
            },
        )
}

#[wrom]
pub fn directive_locations<'a, T, I>() -> impl RecoverableParser<I, DirectiveLocations<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordOn),
        opt(punctuator(TokenKind::Pipe)),
        directive_location().recover(Missing::unary(Diagnostic::missing_first_directive_location)),
        many_ext(
            punctuator(TokenKind::Pipe).and(directive_location().recover(Missing::unary(
                Diagnostic::missing_second_directive_location,
            ))),
        ),
    )
        .map(|(on, pipe, first, locations)| DirectiveLocations {
            on,
            pipe,
            first,
            locations,
        })
}

#[wrom]
pub fn directive_location<'a, T, I>() -> impl RecoverableParser<I, DirectiveLocation<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        executable_directive_location().map(DirectiveLocation::ExecutableDirectiveLocation),
        type_system_directive_location().map(DirectiveLocation::TypeSystemDirectiveLocation),
    ))
}

#[wrom]
pub fn executable_directive_location<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDirectiveLocation<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        keyword(TokenKind::KeywordDirectiveQuery).map(ExecutableDirectiveLocation::Query),
        keyword(TokenKind::KeywordDirectiveMutation).map(ExecutableDirectiveLocation::Mutation),
        keyword(TokenKind::KeywordDirectiveSubscription)
            .map(ExecutableDirectiveLocation::Subscription),
        keyword(TokenKind::KeywordDirectiveField).map(ExecutableDirectiveLocation::Field),
        keyword(TokenKind::KeywordDirectiveFragmentDefinition)
            .map(ExecutableDirectiveLocation::FragmentDefinition),
        keyword(TokenKind::KeywordDirectiveFragmentSpread)
            .map(ExecutableDirectiveLocation::FragmentSpread),
        keyword(TokenKind::KeywordDirectiveInlineFragment)
            .map(ExecutableDirectiveLocation::InlineFragment),
        keyword(TokenKind::KeywordDirectiveVariableDefinition)
            .map(ExecutableDirectiveLocation::VariableDefinition),
    ))
}

#[wrom]
pub fn type_system_directive_location<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDirectiveLocation<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        keyword(TokenKind::KeywordDirectiveSchema).map(TypeSystemDirectiveLocation::Schema),
        keyword(TokenKind::KeywordDirectiveScalar).map(TypeSystemDirectiveLocation::Scalar),
        keyword(TokenKind::KeywordDirectiveObject).map(TypeSystemDirectiveLocation::Object),
        keyword(TokenKind::KeywordDirectiveFieldDefinition)
            .map(TypeSystemDirectiveLocation::FieldDefinition),
        keyword(TokenKind::KeywordDirectiveArgumentDefinition)
            .map(TypeSystemDirectiveLocation::ArgumentDefinition),
        keyword(TokenKind::KeywordDirectiveInterface).map(TypeSystemDirectiveLocation::Interface),
        keyword(TokenKind::KeywordDirectiveUnion).map(TypeSystemDirectiveLocation::Union),
        keyword(TokenKind::KeywordDirectiveEnum).map(TypeSystemDirectiveLocation::Enum),
        keyword(TokenKind::KeywordDirectiveEnumValue).map(TypeSystemDirectiveLocation::EnumValue),
        keyword(TokenKind::KeywordDirectiveInputObject)
            .map(TypeSystemDirectiveLocation::InputObject),
        keyword(TokenKind::KeywordDirectiveInputFieldDefinition)
            .map(TypeSystemDirectiveLocation::InputFieldDefinition),
    ))
}

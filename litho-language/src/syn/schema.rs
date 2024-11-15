use litho_diagnostics::Diagnostic;
use wrom::{alt, delimited, many0, many1, opt, Input, RecoverableParser};
use wrom_derive::wrom;

use crate::ast::*;
use crate::lex::Token;

use super::combinators::{keyword, name, name_unless, punctuator, string_value};
use super::executable::{default_value, directives, enum_value, named_type, operation_type, ty};
use super::{Error, RECURSION_LIMIT};

pub fn type_system_document<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDocument<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many0(type_system_definition()).map(|definitions| TypeSystemDocument { definitions })
}

#[wrom(schema_definition().or(type_definition()).or(directive_definition()))]
pub fn type_system_definition<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        schema_definition().map(TypeSystemDefinition::SchemaDefinition),
        type_definition()
            .map(Into::into)
            .map(TypeSystemDefinition::TypeDefinition),
        directive_definition()
            .map(Into::into)
            .map(TypeSystemDefinition::DirectiveDefinition),
    ))
}

#[wrom(type_system_definition_or_extension())]
pub fn type_system_extension_document<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemExtensionDocument<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many0(type_system_definition_or_extension())
        .map(|definitions| TypeSystemExtensionDocument { definitions })
}

#[wrom(type_system_definition().or(type_system_extension()))]
pub fn type_system_definition_or_extension<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDefinitionOrExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        type_system_definition().map(TypeSystemDefinitionOrExtension::TypeSystemDefinition),
        type_system_extension().map(TypeSystemDefinitionOrExtension::TypeSystemExtension),
    ))
}

#[wrom(schema_extension().or(type_extension()))]
pub fn type_system_extension<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        schema_extension().map(TypeSystemExtension::SchemaExtension),
        type_extension()
            .map(Into::into)
            .map(TypeSystemExtension::TypeExtension),
    ))
}

#[wrom(string_value())]
pub fn description<'a, T, I>() -> impl RecoverableParser<I, Description<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    string_value().map(Description)
}

#[wrom(keyword("schema"))]
pub fn schema_definition<'a, T, I>() -> impl RecoverableParser<I, SchemaDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("schema"),
        opt(directives()),
        root_operation_type_definitions().recover(Missing::unary(
            Diagnostic::missing_root_operation_type_definitions,
        )),
    )
        .map(|(schema, directives, type_definitions)| SchemaDefinition {
            description: None,
            schema,
            directives,
            type_definitions,
        })
}

#[wrom(punctuator("{"))]
pub fn root_operation_type_definitions<'a, T, I>(
) -> impl RecoverableParser<I, RootOperationTypeDefinitions<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator("{"),
        many1(root_operation_type_definition()).recover(Missing::unary(
            Diagnostic::missing_root_operation_type_definitions,
        )),
        punctuator("}"),
        Missing::binary(Diagnostic::missing_root_operation_type_definitions_closing_brace),
    )
    .map(|(left, definitions, right)| RootOperationTypeDefinitions {
        braces: (left, right),
        definitions,
    })
}

#[wrom(operation_type())]
pub fn root_operation_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, RootOperationTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        operation_type(),
        punctuator(":").recover(Missing::unary(
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

#[wrom(keyword("extend"))]
pub fn schema_extension<'a, T, I>() -> impl RecoverableParser<I, SchemaExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("schema"),
        opt(directives()),
        opt(root_operation_type_definitions()),
    )
        .map(
            |(extend, schema, directives, type_definitions)| SchemaExtension {
                extend_schema: (extend, schema),
                directives,
                type_definitions,
            },
        )
}

#[wrom(keyword("scalar").or(keyword("type")).or(keyword("interface")).or(keyword("union")).or(keyword("enum")).or(keyword("input")))]
pub fn type_definition<'a, T, I>() -> impl RecoverableParser<I, TypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        scalar_type_definition().map(TypeDefinition::ScalarTypeDefinition),
        object_type_definition().map(TypeDefinition::ObjectTypeDefinition),
        interface_type_definition().map(TypeDefinition::InterfaceTypeDefinition),
        union_type_definition().map(TypeDefinition::UnionTypeDefinition),
        enum_type_definition().map(TypeDefinition::EnumTypeDefinition),
        input_object_type_definition().map(TypeDefinition::InputObjectTypeDefinition),
    ))
}

#[wrom(keyword("extend"))]
pub fn type_extension<'a, T, I>() -> impl RecoverableParser<I, TypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        scalar_type_extension().map(TypeExtension::ScalarTypeExtension),
        object_type_extension().map(TypeExtension::ObjectTypeExtension),
        interface_type_extension().map(TypeExtension::InterfaceTypeExtension),
        union_type_extension().map(TypeExtension::UnionTypeExtension),
        enum_type_extension().map(TypeExtension::EnumTypeExtension),
        input_object_type_extension().map(TypeExtension::InputObjectTypeExtension),
    ))
}

#[wrom(keyword("scalar"))]
pub fn scalar_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, ScalarTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("scalar"),
        name().recover(Missing::unary(
            Diagnostic::missing_scalar_type_definition_name,
        )),
        opt(directives()),
    )
        .map(|(scalar, name, directives)| ScalarTypeDefinition {
            description: None,
            scalar,
            name,
            directives,
        })
}

#[wrom(keyword("extend"))]
pub fn scalar_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, ScalarTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("scalar"),
        named_type().recover(Missing::unary(
            Diagnostic::missing_scalar_type_extension_name,
        )),
        directives().recover(Missing::unary(
            Diagnostic::missing_scalar_type_extension_directives,
        )),
    )
        .map(|(extend, scalar, name, directives)| ScalarTypeExtension {
            extend_scalar: (extend, scalar),
            name,
            directives,
        })
}

#[wrom(keyword("type"))]
pub fn object_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, ObjectTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("type"),
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
                    description: None,
                    ty,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom(keyword("implements"))]
pub fn implements_interfaces<'a, T, I>(
) -> impl RecoverableParser<I, ImplementsInterfaces<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("implements"),
        opt(punctuator("&")),
        named_type().map(Into::into).recover(Missing::unary(
            Diagnostic::missing_first_implements_interface,
        )),
        many0(
            punctuator("&").and(named_type().map(Into::into).recover(Missing::unary(
                Diagnostic::missing_second_implements_interface,
            ))),
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

#[wrom(punctuator("{"))]
pub fn fields_definition<'a, T, I>() -> impl RecoverableParser<I, FieldsDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator("{"),
        many0(field_definition().map(Into::into)),
        punctuator("}"),
        Missing::binary(Diagnostic::missing_fields_definition_closing_brace),
    )
    .map(|(left, definitions, right)| FieldsDefinition {
        braces: (left, right),
        definitions,
    })
}

#[wrom(description().or(name()))]
pub fn field_definition<'a, T, I>() -> impl RecoverableParser<I, FieldDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        opt(description()),
        name(),
        opt(arguments_definition().map(Into::into)),
        punctuator(":").recover(Missing::unary(Diagnostic::missing_field_definition_colon)),
        ty(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_field_definition_type)),
        opt(directives()),
    )
        .map(
            |(description, name, arguments_definition, colon, ty, directives)| FieldDefinition {
                description,
                name,
                arguments_definition,
                colon,
                ty,
                directives,
            },
        )
}

#[wrom(punctuator("("))]
pub fn arguments_definition<'a, T, I>(
) -> impl RecoverableParser<I, ArgumentsDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator("("),
        many0(input_value_definition().map(Into::into)),
        punctuator(")"),
        Missing::binary(Diagnostic::missing_arguments_definition_closing_parenthesis),
    )
    .map(|(left, definitions, right)| ArgumentsDefinition {
        parens: (left, right),
        definitions,
    })
}

#[wrom(description().or(name()))]
pub fn input_value_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputValueDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        opt(description()),
        name(),
        punctuator(":").recover(Missing::unary(
            Diagnostic::missing_input_value_definition_colon,
        )),
        ty(RECURSION_LIMIT).recover(Missing::unary(
            Diagnostic::missing_input_value_definition_type,
        )),
        opt(default_value()),
        opt(directives()),
    )
        .map(
            |(description, name, colon, ty, default_value, directives)| InputValueDefinition {
                description,
                name,
                colon,
                ty,
                default_value,
                directives,
            },
        )
}

#[wrom(keyword("extend"))]
pub fn object_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, ObjectTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("type"),
        named_type().recover(Missing::unary(
            Diagnostic::missing_object_type_extension_name,
        )),
        opt(implements_interfaces()),
        opt(directives()),
        opt(fields_definition()),
    )
        .map(
            |(extend, ty, name, implements_interfaces, directives, fields_definition)| {
                ObjectTypeExtension {
                    extend_type: (extend, ty),
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom(keyword("interface"))]
pub fn interface_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, InterfaceTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("interface"),
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
                    description: None,
                    interface,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom(keyword("extend"))]
pub fn interface_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, InterfaceTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("interface"),
        named_type().recover(Missing::unary(
            Diagnostic::missing_interface_type_extension_name,
        )),
        opt(implements_interfaces()),
        opt(directives()),
        opt(fields_definition()),
    )
        .map(
            |(extend, interface, name, implements_interfaces, directives, fields_definition)| {
                InterfaceTypeExtension {
                    extend_interface: (extend, interface),
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                }
            },
        )
}

#[wrom(keyword("union"))]
pub fn union_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, UnionTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("union"),
        name().recover(Missing::unary(
            Diagnostic::missing_union_type_definition_name,
        )),
        opt(directives()),
        opt(union_member_types()),
    )
        .map(
            |(union_kw, name, directives, member_types)| UnionTypeDefinition {
                description: None,
                union_kw,
                name,
                directives,
                member_types,
            },
        )
}

#[wrom(punctuator("="))]
pub fn union_member_types<'a, T, I>() -> impl RecoverableParser<I, UnionMemberTypes<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        punctuator("="),
        opt(punctuator("|")),
        named_type()
            .map(Into::into)
            .recover(Missing::unary(Diagnostic::missing_first_union_member_type)),
        many0(
            punctuator("|").and(
                named_type()
                    .map(Into::into)
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

#[wrom(keyword("extend"))]
pub fn union_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, UnionTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("union"),
        named_type().recover(Missing::unary(
            Diagnostic::missing_union_type_extension_name,
        )),
        opt(directives()),
        opt(union_member_types()),
    )
        .map(
            |(extend, union_kw, name, directives, member_types)| UnionTypeExtension {
                extend_union: (extend, union_kw),
                name,
                directives,
                member_types,
            },
        )
}

#[wrom(punctuator("enum"))]
pub fn enum_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("enum"),
        name().recover(Missing::unary(
            Diagnostic::missing_enum_type_definition_name,
        )),
        opt(directives()),
        opt(enum_values_definition()),
    )
        .map(
            |(enum_kw, name, directives, values_definition)| EnumTypeDefinition {
                description: None,
                enum_kw,
                name,
                directives,
                values_definition,
            },
        )
}

#[wrom(punctuator("{"))]
pub fn enum_values_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumValuesDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator("{"),
        many0(enum_value_definition().map(Into::into)),
        punctuator("}"),
        Missing::binary(Diagnostic::missing_enum_values_closing_brace),
    )
    .map(|(left, definitions, right)| EnumValuesDefinition {
        braces: (left, right),
        definitions,
    })
}

#[wrom(description().or(enum_value()))]
pub fn enum_value_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumValueDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
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

#[wrom(keyword("extend"))]
pub fn enum_type_extension<'a, T, I>() -> impl RecoverableParser<I, EnumTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("enum"),
        named_type().recover(Missing::unary(Diagnostic::missing_enum_type_extension_name)),
        opt(directives()),
        opt(enum_values_definition()),
    )
        .map(
            |(extend, enum_kw, name, directives, values_definition)| EnumTypeExtension {
                extend_enum: (extend, enum_kw),
                name,
                directives,
                values_definition,
            },
        )
}

#[wrom(keyword("input"))]
pub fn input_object_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputObjectTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("input"),
        name().recover(Missing::unary(
            Diagnostic::missing_input_object_type_definition_name,
        )),
        opt(directives()),
        opt(input_fields_definition()),
    )
        .map(
            |(input, name, directives, fields_definition)| InputObjectTypeDefinition {
                description: None,
                input,
                name,
                directives,
                fields_definition,
            },
        )
}

#[wrom(punctuator("{"))]
pub fn input_fields_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputFieldsDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator("{"),
        many0(input_value_definition().map(Into::into)),
        punctuator("}"),
        Missing::binary(Diagnostic::missing_input_fields_definition_closing_brace),
    )
    .map(|(left, definitions, right)| InputFieldsDefinition {
        braces: (left, right),
        definitions,
    })
}

#[wrom(keyword("extend"))]
pub fn input_object_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, InputObjectTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("extend"),
        keyword("input"),
        named_type().recover(Missing::unary(
            Diagnostic::missing_input_object_type_extension_name,
        )),
        opt(directives()),
        opt(input_fields_definition()),
    )
        .map(
            |(extend, input, name, directives, fields_definition)| InputObjectTypeExtension {
                extend_input: (extend, input),
                name,
                directives,
                fields_definition,
            },
        )
}

#[wrom(keyword("directive"))]
pub fn directive_definition<'a, T, I>(
) -> impl RecoverableParser<I, DirectiveDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("directive"),
        punctuator("@").recover(Missing::unary(Diagnostic::missing_directive_definition_at)),
        name_unless("on").recover(Missing::unary(
            Diagnostic::missing_directive_definition_name,
        )),
        opt(arguments_definition().map(Into::into)),
        opt(keyword("repeatable")),
        directive_locations().recover(Missing::unary(
            Diagnostic::missing_directive_definition_locations,
        )),
    )
        .map(
            |(directive, at, name, arguments_definition, repeatable, locations)| {
                DirectiveDefinition {
                    description: None,
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

#[wrom(keyword("on"))]
pub fn directive_locations<'a, T, I>(
) -> impl RecoverableParser<I, DirectiveLocations<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword("on"),
        opt(punctuator("|")),
        directive_location().recover(Missing::unary(Diagnostic::missing_first_directive_location)),
        many0(
            punctuator("|").and(directive_location().recover(Missing::unary(
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

pub fn directive_location<'a, T, I>() -> impl RecoverableParser<I, DirectiveLocation<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        executable_directive_location().map(DirectiveLocation::ExecutableDirectiveLocation),
        type_system_directive_location().map(DirectiveLocation::TypeSystemDirectiveLocation),
    ))
}

pub fn executable_directive_location<'a, T, I>() -> wrom::Alt<
    [wrom::Map<
        impl RecoverableParser<I, Name<T>, Error>,
        Name<T>,
        fn(Name<T>) -> ExecutableDirectiveLocation<T>,
    >; 8],
>
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt([
        keyword("QUERY").map(ExecutableDirectiveLocation::Query),
        keyword("MUTATION").map(ExecutableDirectiveLocation::Mutation),
        keyword("SUBSCRIPTION").map(ExecutableDirectiveLocation::Subscription),
        keyword("FIELD").map(ExecutableDirectiveLocation::Field),
        keyword("FRAGMENT_DEFINITION").map(ExecutableDirectiveLocation::FragmentDefinition),
        keyword("FRAGMENT_SPREAD").map(ExecutableDirectiveLocation::FragmentSpread),
        keyword("INLINE_FRAGMENT").map(ExecutableDirectiveLocation::InlineFragment),
        keyword("VARIABLE_DEFINITION").map(ExecutableDirectiveLocation::VariableDefinition),
    ])
}

pub fn type_system_directive_location<'a, T, I>() -> wrom::Alt<
    [wrom::Map<
        impl RecoverableParser<I, Name<T>, Error>,
        Name<T>,
        fn(Name<T>) -> TypeSystemDirectiveLocation<T>,
    >; 11],
>
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt([
        keyword("SCHEMA").map(TypeSystemDirectiveLocation::Schema),
        keyword("SCALAR").map(TypeSystemDirectiveLocation::Scalar),
        keyword("OBJECT").map(TypeSystemDirectiveLocation::Object),
        keyword("FIELD_DEFINITION").map(TypeSystemDirectiveLocation::FieldDefinition),
        keyword("ARGUMENT_DEFINITION").map(TypeSystemDirectiveLocation::ArgumentDefinition),
        keyword("INTERFACE").map(TypeSystemDirectiveLocation::Interface),
        keyword("UNION").map(TypeSystemDirectiveLocation::Union),
        keyword("ENUM").map(TypeSystemDirectiveLocation::Enum),
        keyword("ENUM_VALUE").map(TypeSystemDirectiveLocation::EnumValue),
        keyword("INPUT_OBJECT").map(TypeSystemDirectiveLocation::InputObject),
        keyword("INPUT_FIELD_DEFINITION").map(TypeSystemDirectiveLocation::InputFieldDefinition),
    ])
}

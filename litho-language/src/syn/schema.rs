use litho_diagnostics::Diagnostic;
use wrom::branch::alt;
use wrom::combinator::opt;
use wrom::multi::{many0, many1};
use wrom::sequence::delimited;
use wrom::{Input, RecoverableParser};

use crate::ast::*;
use crate::lex::Token;

use super::combinators::{keyword, name, name_unless, punctuator, string_value};
use super::executable::{default_value, directives, enum_value, named_type, operation_type, ty};
use super::Error;

pub fn type_system_document<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDocument<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        many0(type_system_definition()).map(|definitions| TypeSystemDocument { definitions })
    })
}

pub fn type_system_definition<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            schema_definition().map(TypeSystemDefinition::SchemaDefinition),
            type_definition()
                .map(Into::into)
                .map(TypeSystemDefinition::TypeDefinition),
            directive_definition()
                .map(Into::into)
                .map(TypeSystemDefinition::DirectiveDefinition),
        ))
    })
}

pub fn type_system_extension_document<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemExtensionDocument<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        many0(type_system_definition_or_extension())
            .map(|definitions| TypeSystemExtensionDocument { definitions })
    })
}

pub fn type_system_definition_or_extension<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDefinitionOrExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            type_system_definition().map(TypeSystemDefinitionOrExtension::TypeSystemDefinition),
            type_system_extension().map(TypeSystemDefinitionOrExtension::TypeSystemExtension),
        ))
    })
}

pub fn type_system_extension<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            schema_extension().map(TypeSystemExtension::SchemaExtension),
            type_extension()
                .map(Into::into)
                .map(TypeSystemExtension::TypeExtension),
        ))
    })
}

pub fn description<'a, T, I>() -> impl RecoverableParser<I, Description<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| string_value().map(Description))
}

pub fn schema_definition<'a, T, I>() -> impl RecoverableParser<I, SchemaDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("schema"))
            .and(opt(directives()))
            .and(root_operation_type_definitions().recover(Missing::unary(
                Diagnostic::missing_root_operation_type_definitions,
            )))
            .flatten()
            .map(
                |(description, schema, directives, type_definitions)| SchemaDefinition {
                    description,
                    schema,
                    directives,
                    type_definitions,
                },
            )
    })
}

pub fn root_operation_type_definitions<'a, T, I>(
) -> impl RecoverableParser<I, RootOperationTypeDefinitions<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
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
    })
}

pub fn root_operation_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, RootOperationTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        operation_type()
            .and(punctuator(":").recover(Missing::unary(
                Diagnostic::missing_root_operation_type_definition_colon,
            )))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_root_operation_type_definition_named_type,
            )))
            .flatten()
            .map(
                |(operation_type, colon, named_type)| RootOperationTypeDefinition {
                    operation_type,
                    colon,
                    named_type,
                },
            )
    })
}

pub fn schema_extension<'a, T, I>() -> impl RecoverableParser<I, SchemaExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("schema"))
            .and(opt(directives()))
            .and(opt(root_operation_type_definitions()))
            .flatten()
            .map(
                |(extend, schema, directives, type_definitions)| SchemaExtension {
                    extend_schema: (extend, schema),
                    directives,
                    type_definitions,
                },
            )
    })
}

pub fn type_definition<'a, T, I>() -> impl RecoverableParser<I, TypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            scalar_type_definition().map(TypeDefinition::ScalarTypeDefinition),
            object_type_definition().map(TypeDefinition::ObjectTypeDefinition),
            interface_type_definition().map(TypeDefinition::InterfaceTypeDefinition),
            union_type_definition().map(TypeDefinition::UnionTypeDefinition),
            enum_type_definition().map(TypeDefinition::EnumTypeDefinition),
            input_object_type_definition().map(TypeDefinition::InputObjectTypeDefinition),
        ))
    })
}

pub fn type_extension<'a, T, I>() -> impl RecoverableParser<I, TypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            scalar_type_extension().map(TypeExtension::ScalarTypeExtension),
            object_type_extension().map(TypeExtension::ObjectTypeExtension),
            interface_type_extension().map(TypeExtension::InterfaceTypeExtension),
            union_type_extension().map(TypeExtension::UnionTypeExtension),
            enum_type_extension().map(TypeExtension::EnumTypeExtension),
            input_object_type_extension().map(TypeExtension::InputObjectTypeExtension),
        ))
    })
}

pub fn scalar_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, ScalarTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("scalar"))
            .and(name().recover(Missing::unary(
                Diagnostic::missing_scalar_type_definition_name,
            )))
            .and(opt(directives()))
            .flatten()
            .map(
                |(description, scalar, name, directives)| ScalarTypeDefinition {
                    description,
                    scalar,
                    name,
                    directives,
                },
            )
    })
}

pub fn scalar_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, ScalarTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("scalar"))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_scalar_type_extension_name,
            )))
            .and(directives().recover(Missing::unary(
                Diagnostic::missing_scalar_type_extension_directives,
            )))
            .flatten()
            .map(|(extend, scalar, name, directives)| ScalarTypeExtension {
                extend_scalar: (extend, scalar),
                name,
                directives,
            })
    })
}

pub fn object_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, ObjectTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("type"))
            .and(name().recover(Missing::unary(
                Diagnostic::missing_object_type_definition_name,
            )))
            .and(opt(implements_interfaces()))
            .and(opt(directives()))
            .and(opt(fields_definition()))
            .flatten()
            .map(
                |(description, ty, name, implements_interfaces, directives, fields_definition)| {
                    ObjectTypeDefinition {
                        description,
                        ty,
                        name,
                        implements_interfaces,
                        directives,
                        fields_definition,
                    }
                },
            )
    })
}

pub fn implements_interfaces<'a, T, I>(
) -> impl RecoverableParser<I, ImplementsInterfaces<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("implements")
            .and(opt(punctuator("&")))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_first_implements_interface,
            )))
            .and(many0(punctuator("&").and(named_type().recover(
                Missing::unary(Diagnostic::missing_second_implements_interface),
            ))))
            .flatten()
            .map(
                |(implements, ampersand, first, types)| ImplementsInterfaces {
                    implements,
                    ampersand,
                    first,
                    types,
                },
            )
    })
}

pub fn fields_definition<'a, T, I>() -> impl RecoverableParser<I, FieldsDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
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
    })
}

pub fn field_definition<'a, T, I>() -> impl RecoverableParser<I, FieldDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(name())
            .and(opt(arguments_definition().map(Into::into)))
            .and(
                punctuator(":").recover(Missing::unary(Diagnostic::missing_field_definition_colon)),
            )
            .and(ty().recover(Missing::unary(Diagnostic::missing_field_definition_type)))
            .and(opt(directives()))
            .flatten()
            .map(
                |(description, name, arguments_definition, colon, ty, directives)| {
                    FieldDefinition {
                        description,
                        name,
                        arguments_definition,
                        colon,
                        ty,
                        directives,
                    }
                },
            )
    })
}

pub fn arguments_definition<'a, T, I>(
) -> impl RecoverableParser<I, ArgumentsDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
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
    })
}

pub fn input_value_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputValueDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(name())
            .and(punctuator(":").recover(Missing::unary(
                Diagnostic::missing_input_value_definition_colon,
            )))
            .and(ty().recover(Missing::unary(
                Diagnostic::missing_input_value_definition_type,
            )))
            .and(opt(default_value()))
            .and(opt(directives()))
            .flatten()
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
    })
}

pub fn object_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, ObjectTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("type"))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_object_type_extension_name,
            )))
            .and(opt(implements_interfaces()))
            .and(opt(directives()))
            .and(opt(fields_definition()))
            .flatten()
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
    })
}

pub fn interface_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, InterfaceTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("interface"))
            .and(name().recover(Missing::unary(
                Diagnostic::missing_interface_type_definition_name,
            )))
            .and(opt(implements_interfaces()))
            .and(opt(directives()))
            .and(opt(fields_definition()))
            .flatten()
            .map(
                |(
                    description,
                    interface,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                )| InterfaceTypeDefinition {
                    description,
                    interface,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                },
            )
    })
}

pub fn interface_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, InterfaceTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("interface"))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_interface_type_extension_name,
            )))
            .and(opt(implements_interfaces()))
            .and(opt(directives()))
            .and(opt(fields_definition()))
            .flatten()
            .map(
                |(
                    extend,
                    interface,
                    name,
                    implements_interfaces,
                    directives,
                    fields_definition,
                )| {
                    InterfaceTypeExtension {
                        extend_interface: (extend, interface),
                        name,
                        implements_interfaces,
                        directives,
                        fields_definition,
                    }
                },
            )
    })
}

pub fn union_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, UnionTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("union"))
            .and(name().recover(Missing::unary(
                Diagnostic::missing_union_type_definition_name,
            )))
            .and(opt(directives()))
            .and(opt(union_member_types()))
            .flatten()
            .map(
                |(description, union_kw, name, directives, member_types)| UnionTypeDefinition {
                    description,
                    union_kw,
                    name,
                    directives,
                    member_types,
                },
            )
    })
}

pub fn union_member_types<'a, T, I>() -> impl RecoverableParser<I, UnionMemberTypes<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("=")
            .and(opt(punctuator("|")))
            .and(named_type().recover(Missing::unary(Diagnostic::missing_first_union_member_type)))
            .and(many0(punctuator("|").and(named_type().recover(
                Missing::unary(Diagnostic::missing_second_union_member_type),
            ))))
            .flatten()
            .map(|(eq, pipe, first, types)| UnionMemberTypes {
                eq,
                pipe,
                first,
                types,
            })
    })
}

pub fn union_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, UnionTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("union"))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_union_type_extension_name,
            )))
            .and(opt(directives()))
            .and(opt(union_member_types()))
            .flatten()
            .map(
                |(extend, union_kw, name, directives, member_types)| UnionTypeExtension {
                    extend_union: (extend, union_kw),
                    name,
                    directives,
                    member_types,
                },
            )
    })
}

pub fn enum_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("enum"))
            .and(name().recover(Missing::unary(
                Diagnostic::missing_enum_type_definition_name,
            )))
            .and(opt(directives()))
            .and(opt(enum_values_definition()))
            .flatten()
            .map(
                |(description, enum_kw, name, directives, values_definition)| EnumTypeDefinition {
                    description,
                    enum_kw,
                    name,
                    directives,
                    values_definition,
                },
            )
    })
}

pub fn enum_values_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumValuesDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        delimited(
            punctuator("{"),
            many0(enum_value_definition()),
            punctuator("}"),
            Missing::binary(Diagnostic::missing_enum_values_closing_brace),
        )
        .map(|(left, definitions, right)| EnumValuesDefinition {
            braces: (left, right),
            definitions,
        })
    })
}

pub fn enum_value_definition<'a, T, I>(
) -> impl RecoverableParser<I, EnumValueDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(enum_value())
            .and(opt(directives()))
            .flatten()
            .map(
                |(description, enum_value, directives)| EnumValueDefinition {
                    description,
                    enum_value,
                    directives,
                },
            )
    })
}

pub fn enum_type_extension<'a, T, I>() -> impl RecoverableParser<I, EnumTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("enum"))
            .and(named_type().recover(Missing::unary(Diagnostic::missing_enum_type_extension_name)))
            .and(opt(directives()))
            .and(opt(enum_values_definition()))
            .flatten()
            .map(
                |(extend, enum_kw, name, directives, values_definition)| EnumTypeExtension {
                    extend_enum: (extend, enum_kw),
                    name,
                    directives,
                    values_definition,
                },
            )
    })
}

pub fn input_object_type_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputObjectTypeDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and_recognize(keyword("input"))
            .and(name().recover(Missing::unary(
                Diagnostic::missing_input_object_type_definition_name,
            )))
            .and(opt(directives()))
            .and(opt(input_fields_definition()))
            .flatten()
            .map(
                |(description, input, name, directives, fields_definition)| {
                    InputObjectTypeDefinition {
                        description,
                        input,
                        name,
                        directives,
                        fields_definition,
                    }
                },
            )
    })
}

pub fn input_fields_definition<'a, T, I>(
) -> impl RecoverableParser<I, InputFieldsDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
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
    })
}

pub fn input_object_type_extension<'a, T, I>(
) -> impl RecoverableParser<I, InputObjectTypeExtension<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("extend")
            .and(keyword("input"))
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_input_object_type_extension_name,
            )))
            .and(opt(directives()))
            .and(opt(input_fields_definition()))
            .flatten()
            .map(
                |(extend, input, name, directives, fields_definition)| InputObjectTypeExtension {
                    extend_input: (extend, input),
                    name,
                    directives,
                    fields_definition,
                },
            )
    })
}

pub fn directive_definition<'a, T, I>(
) -> impl RecoverableParser<I, DirectiveDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        opt(description())
            .and(keyword("directive"))
            .and(
                punctuator("@")
                    .recover(Missing::unary(Diagnostic::missing_directive_definition_at)),
            )
            .and(name_unless("on").recover(Missing::unary(
                Diagnostic::missing_directive_definition_name,
            )))
            .and(opt(arguments_definition().map(Into::into)))
            .and(opt(keyword("repeatable")))
            .and(directive_locations().recover(Missing::unary(
                Diagnostic::missing_directive_definition_locations,
            )))
            .flatten()
            .map(
                |(
                    description,
                    directive,
                    at,
                    name,
                    arguments_definition,
                    repeatable,
                    locations,
                )| {
                    DirectiveDefinition {
                        description,
                        directive,
                        at,
                        name,
                        arguments_definition,
                        repeatable,
                        locations,
                    }
                },
            )
    })
}

pub fn directive_locations<'a, T, I>(
) -> impl RecoverableParser<I, DirectiveLocations<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("on")
            .and(opt(punctuator("|")))
            .and(
                directive_location()
                    .recover(Missing::unary(Diagnostic::missing_first_directive_location)),
            )
            .and(many0(punctuator("|").and(directive_location().recover(
                Missing::unary(Diagnostic::missing_second_directive_location),
            ))))
            .flatten()
            .map(|(on, pipe, first, locations)| DirectiveLocations {
                on,
                pipe,
                first,
                locations,
            })
    })
}

pub fn directive_location<'a, T, I>() -> impl RecoverableParser<I, DirectiveLocation<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            executable_directive_location().map(DirectiveLocation::ExecutableDirectiveLocation),
            type_system_directive_location().map(DirectiveLocation::TypeSystemDirectiveLocation),
        ))
    })
}

pub fn executable_directive_location<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDirectiveLocation<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            keyword("QUERY").map(ExecutableDirectiveLocation::Query),
            keyword("MUTATION").map(ExecutableDirectiveLocation::Mutation),
            keyword("SUBSCRIPTION").map(ExecutableDirectiveLocation::Subscription),
            keyword("FIELD").map(ExecutableDirectiveLocation::Field),
            keyword("FRAGMENT_DEFINITION").map(ExecutableDirectiveLocation::FragmentDefinition),
            keyword("FRAGMENT_SPREAD").map(ExecutableDirectiveLocation::FragmentSpread),
            keyword("VARIABLE_DEFINITION").map(ExecutableDirectiveLocation::VariableDefinition),
        ))
    })
}

pub fn type_system_directive_location<'a, T, I>(
) -> impl RecoverableParser<I, TypeSystemDirectiveLocation<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
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
            keyword("INPUT_FIELD_DEFINITION")
                .map(TypeSystemDirectiveLocation::InputFieldDefinition),
        ))
    })
}

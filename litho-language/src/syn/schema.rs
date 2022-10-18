use wrom::branch::alt;
use wrom::combinator::opt;
use wrom::multi::many0;
use wrom::sequence::delimited;
use wrom::{Input, RecoverableParser};

use crate::ast::*;
use crate::lex::Token;

use super::combinators::{keyword, name, punctuator, string_value};
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
            .and(
                root_operation_type_definitions().recover(|| {
                    "Schema definition is missing root operation type definitions.".into()
                }),
            )
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
            many0(root_operation_type_definition()),
            punctuator("}"),
            Missing::delimiter_complaint(
                "Root operation type definitions are missing closing delimiter.",
                "This `{` here ...",
                "... should have a corresponding `}` here.",
            ),
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
            .and(punctuator(":").recover(|| "Missing colon.".into()))
            .and(named_type().recover(|| "Missing named type.".into()))
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
            .and(name().recover(|| "Scalar type definition is missing a name".into()))
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
            .and(name().recover(|| "Scalar extension is missing name.".into()))
            .and(directives().recover(|| "Scalar extension is missing directives.".into()))
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
            .and(name().recover(|| "Object type definition is missing a name.".into()))
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
            .and(named_type().recover(|| "Missing name of an interface here.".into()))
            .and(many0(punctuator("&").and(named_type().recover(|| {
                "Expected the name of an interface here.".into()
            }))))
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
            Missing::delimiter_complaint(
                "Fields definition is missing `}` delimiter.",
                "This `{` here ...",
                "... should have a corresponding `}` here.",
            ),
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
            .and(punctuator(":").recover(|| "Field is missing a colon here.".into()))
            .and(ty().recover(|| "Field is missing a type here.".into()))
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
            Missing::delimiter_complaint(
                "Arguments are missing `)` delimiter.",
                "This `(` here ...",
                "... should have a corresponding `)` here.",
            ),
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
            .and(punctuator(":").recover(|| "Input value is missing a colon.".into()))
            .and(ty().recover(|| "Input value is missing a type.".into()))
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
            .and(name().recover(|| "Object type extension is missing a name here.".into()))
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
            .and(name().recover(|| "Interface definition is missing a name.".into()))
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
            .and(name().recover(|| "Interface extension is missing a name.".into()))
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
            .and(name().recover(|| "Union definition is missing a name.".into()))
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
            .and(named_type().recover(|| "Expected a named type here.".into()))
            .and(many0(punctuator("|").and(
                named_type().recover(|| "Expected a named type here.".into()),
            )))
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
            .and(name().recover(|| "Union extension is missing a name.".into()))
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
            .and(name().recover(|| "Enum definition is missing a name.".into()))
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
            Missing::delimiter_complaint(
                "Enum values are missing `}` delimiter.",
                "This `{` here ...",
                "... should have a corresponding `}` here.",
            ),
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
            .and(name().recover(|| "Enum extension is missing a name.".into()))
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
            .and(name().recover(|| "Input definition is missing a name.".into()))
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
            Missing::delimiter_complaint(
                "Input fields are missing `}` delimiter.",
                "This `{` here ...",
                "... should have a corresponding `}` here.",
            ),
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
            .and(name().recover(|| "Input extension is missing a name.".into()))
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
            .and(punctuator("@").recover(|| "Directive definition is missing a `@` here.".into()))
            .and(name().recover(|| "Directive definition is missing a name here.".into()))
            .and(opt(arguments_definition().map(Into::into)))
            .and(opt(keyword("repeatable")))
            .and(
                directive_locations()
                    .recover(|| "Directive definition is missing locations here.".into()),
            )
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
            .and(directive_location().recover(|| "Expected a directive location here.".into()))
            .and(many0(punctuator("|").and(
                directive_location().recover(|| "Expected a directive location here.".into()),
            )))
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

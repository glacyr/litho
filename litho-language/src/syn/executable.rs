use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use wrom::branch::alt;
use wrom::combinator::opt;
use wrom::multi::{many0, many1};
use wrom::sequence::delimited;
use wrom::{recursive, Input, RecoverableParser};

use crate::ast::*;
use crate::lex::{Name, Token};

use super::combinators::{
    float_value, int_value, keyword, name, name_unless, punctuator, string_value,
};
use super::Error;

pub fn executable_document<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDocument<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        many0(executable_definition()).map(|definitions| ExecutableDocument { definitions })
    })
}

pub fn executable_definition<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            operation_definition()
                .map(Into::into)
                .map(ExecutableDefinition::OperationDefinition),
            fragment_definition()
                .map(Into::into)
                .map(ExecutableDefinition::FragmentDefinition),
        ))
    })
}

pub fn operation_definition<'a, T, I>(
) -> impl RecoverableParser<I, OperationDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        operation_type()
            .and(opt(name()))
            .and(opt(variable_definitions()))
            .and(opt(directives()))
            .and(selection_set().map(Into::into).recover(Missing::unary(
                Diagnostic::missing_operation_definition_selection_set,
            )))
            .unzip()
            .map(
                |(ty, name, variable_definitions, directives, selection_set)| OperationDefinition {
                    ty: Some(ty),
                    name,
                    variable_definitions,
                    directives,
                    selection_set,
                },
            )
    })
}

pub fn operation_type<'a, T, I>() -> impl RecoverableParser<I, OperationType<T>, Error> + 'a
where
    I: Iterator<Item = Token<T>> + Clone + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            keyword("query").map(OperationType::Query),
            keyword("mutation").map(OperationType::Mutation),
            keyword("subscription").map(OperationType::Subscription),
        ))
    })
}

pub fn selection_set<'a, T, I>() -> impl RecoverableParser<I, SelectionSet<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        delimited(
            punctuator("{"),
            many0(selection()),
            punctuator("}"),
            Missing::binary(Diagnostic::missing_selection_set_closing_brace),
        )
        .map(|(brace_left, selections, brace_right)| SelectionSet {
            braces: (brace_left, brace_right),
            selections,
        })
    })
}

pub fn selection<'a, T, I>() -> impl RecoverableParser<I, Selection<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            fragment_spread().map(Selection::FragmentSpread),
            inline_fragment().map(Selection::InlineFragment),
            field().map(Into::into).map(Selection::Field),
        ))
    })
}

pub fn field<'a, T, I>() -> impl RecoverableParser<I, Field<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            alias()
                .and(name().recover(Missing::unary(Diagnostic::missing_field_name)))
                .and(opt(arguments().map(Into::into)))
                .and(opt(directives()))
                .and(opt(selection_set().map(Into::into)))
                .unzip()
                .map(
                    |(alias, name, arguments, directives, selection_set)| Field {
                        alias: Some(alias),
                        name,
                        arguments,
                        directives,
                        selection_set,
                    },
                ),
            name()
                .and(opt(arguments().map(Into::into)))
                .and(opt(directives()))
                .and(opt(selection_set().map(Into::into)))
                .unzip()
                .map(
                    |(name, arguments, directives, selection_set): (Name<T>, _, _, _)| Field {
                        alias: None,
                        name: name.into(),
                        arguments,
                        directives,
                        selection_set,
                    },
                ),
        ))
    })
}

pub fn alias<'a, T, I>() -> impl RecoverableParser<I, Alias<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        name()
            .and(punctuator(":"))
            .map(|(name, colon)| Alias { name, colon })
    })
}

pub fn arguments<'a, T, I>() -> impl RecoverableParser<I, Arguments<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        delimited(
            punctuator("("),
            many0(argument()),
            punctuator(")"),
            Missing::binary(Diagnostic::missing_arguments_closing_parentheses),
        )
        .map(|(left, items, right)| Arguments {
            parens: (left, right),
            items,
        })
    })
}

pub fn argument<'a, T, I>() -> impl RecoverableParser<I, Arc<Argument<T>>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        name()
            .and(punctuator(":").recover(Missing::unary(Diagnostic::missing_argument_colon)))
            .and(value().recover(Missing::unary(Diagnostic::missing_argument_value)))
            .unzip()
            .map(|(name, colon, value)| Argument { name, colon, value })
            .map(Into::into)
    })
}

pub fn fragment_spread<'a, T, I>() -> impl RecoverableParser<I, Arc<FragmentSpread<T>>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("...")
            .and(name_unless("on"))
            .and(opt(directives()))
            .unzip()
            .map(|(dots, fragment_name, directives)| FragmentSpread {
                dots,
                fragment_name,
                directives,
            })
            .map(Into::into)
    })
}

pub fn inline_fragment<'a, T, I>() -> impl RecoverableParser<I, InlineFragment<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("...")
            .and(opt(type_condition()))
            .and(opt(directives()))
            .and(selection_set().map(Into::into).recover(Missing::unary(
                Diagnostic::missing_inline_fragment_selection_set,
            )))
            .unzip()
            .map(
                |(dots, type_condition, directives, selection_set)| InlineFragment {
                    dots,
                    type_condition,
                    directives,
                    selection_set,
                },
            )
    })
}

pub fn fragment_definition<'a, T, I>(
) -> impl RecoverableParser<I, FragmentDefinition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("fragment")
            .and(name_unless("on").recover(Missing::unary(Diagnostic::missing_fragment_name)))
            .and(
                type_condition()
                    .recover(Missing::unary(Diagnostic::missing_fragment_type_condition)),
            )
            .and(opt(directives()))
            .and(
                selection_set()
                    .map(Into::into)
                    .recover(Missing::Unary(Diagnostic::missing_fragment_selection_set)),
            )
            .unzip()
            .map(
                |(fragment, fragment_name, type_condition, directives, selection_set)| {
                    FragmentDefinition {
                        fragment,
                        fragment_name,
                        type_condition,
                        directives,
                        selection_set,
                    }
                },
            )
    })
}

pub fn type_condition<'a, T, I>() -> impl RecoverableParser<I, TypeCondition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        keyword("on")
            .and(named_type().recover(Missing::unary(
                Diagnostic::missing_type_condition_named_type,
            )))
            .map(|(on, named_type)| TypeCondition { on, named_type })
    })
}

pub fn value<'a, T, I>() -> impl RecoverableParser<I, Arc<Value<T>>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            int_value().map(Value::IntValue),
            float_value().map(Value::FloatValue),
            string_value().map(Value::StringValue),
            boolean_value().map(Value::BooleanValue),
            null_value().map(Value::NullValue),
            enum_value().map(Value::EnumValue),
            variable().map(Value::Variable),
            list_value().map(Value::ListValue),
            object_value().map(Value::ObjectValue),
        ))
    })
    .map(Into::into)
}

pub fn boolean_value<'a, T, I>() -> impl RecoverableParser<I, BooleanValue<T>, Error> + 'a
where
    I: Iterator<Item = Token<T>> + Clone + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            keyword("true").map(BooleanValue::True),
            keyword("false").map(BooleanValue::False),
        ))
    })
}

pub fn null_value<'a, T, I>() -> impl RecoverableParser<I, NullValue<T>, Error> + 'a
where
    I: Iterator<Item = Token<T>> + Clone + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| keyword("null").map(NullValue))
}

pub fn enum_value<'a, T, I>() -> impl RecoverableParser<I, EnumValue<T>, Error> + 'a
where
    I: Iterator<Item = Token<T>> + Clone + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| name().map(EnumValue))
}

pub fn list_value<'a, T, I>() -> impl RecoverableParser<I, ListValue<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    recursive(|| {
        delimited(
            punctuator("["),
            many0(value()),
            punctuator("]"),
            Missing::binary(Diagnostic::missing_list_value_closing_bracket),
        )
        .map(|(left, values, right)| ListValue {
            brackets: (left, right),
            values,
        })
    })
}

pub fn object_value<'a, T, I>() -> impl RecoverableParser<I, ObjectValue<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    recursive(|| {
        delimited(
            punctuator("{"),
            many0(object_field()),
            punctuator("}"),
            Missing::binary(Diagnostic::missing_object_value_closing_brace),
        )
        .map(|(left, object_fields, right)| ObjectValue {
            braces: (left, right),
            object_fields,
        })
    })
}

pub fn object_field<'a, T, I>() -> impl RecoverableParser<I, ObjectField<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        name()
            .and(punctuator(":").recover(Missing::unary(Diagnostic::missing_object_field_colon)))
            .and(value().recover(Missing::unary(Diagnostic::missing_object_field_value)))
            .unzip()
            .map(|(name, colon, value)| ObjectField { name, colon, value })
    })
}

pub fn variable_definitions<'a, T, I>(
) -> impl RecoverableParser<I, VariableDefinitions<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("(")
            .and(many0(variable_definition()))
            .and_recover(punctuator(")"), |(left, _)| {
                Missing::binary(Diagnostic::missing_variable_definitions_closing_parenthesis)(left)
            })
            .unzip()
            .map(|(left, variable_definitions, right)| VariableDefinitions {
                parens: (left, right),
                variable_definitions,
            })
    })
}

pub fn variable_definition<'a, T, I>(
) -> impl RecoverableParser<I, Arc<VariableDefinition<T>>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        variable()
            .and(punctuator(":").recover(Missing::unary(
                Diagnostic::missing_variable_definition_colon,
            )))
            .and(ty().recover(Missing::unary(Diagnostic::missing_variable_definition_type)))
            .and(opt(default_value()))
            .and(opt(directives()))
            .unzip()
            .map(
                |(variable, colon, ty, default_value, directives)| VariableDefinition {
                    variable,
                    colon,
                    ty,
                    default_value,
                    directives,
                },
            )
            .map(Into::into)
    })
}

pub fn variable<'a, T, I>() -> impl RecoverableParser<I, Variable<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("$")
            .and(name())
            .map(|(dollar, name)| Variable { dollar, name })
    })
}

pub fn default_value<'a, T, I>() -> impl RecoverableParser<I, DefaultValue<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("=")
            .and(value().recover(Missing::unary(Diagnostic::missing_default_value)))
            .map(|(eq, value)| DefaultValue { eq, value })
    })
}

pub fn ty<'a, T, I>() -> impl RecoverableParser<I, Arc<Type<T>>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((
            non_null_type().map(Type::NonNull),
            named_type().map(Type::Named),
            list_type().map(Type::List),
        ))
    })
    .map(Into::into)
}

pub fn named_type<'a, T, I>() -> impl RecoverableParser<I, NamedType<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| name().map(NamedType))
}

pub fn list_type<'a, T, I>() -> impl RecoverableParser<I, ListType<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        delimited(
            punctuator("["),
            ty().recover(Missing::unary(Diagnostic::missing_list_type_wrapped_type)),
            punctuator("]"),
            Missing::binary(Diagnostic::missing_list_type_closing_bracket),
        )
        .map(|(left, ty, right)| ListType {
            brackets: (left, right),
            ty,
        })
    })
}

pub fn non_null_type<'a, T, I>() -> impl RecoverableParser<I, NonNullType<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        alt((named_type().map(Type::Named), list_type().map(Type::List)))
            .map(Into::into)
            .and(punctuator("!"))
            .map(|(ty, bang)| NonNullType { ty, bang })
    })
}

pub fn directives<'a, T, I>() -> impl RecoverableParser<I, Directives<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| many1(directive()).map(|directives| Directives { directives }))
}

pub fn directive<'a, T, I>() -> impl RecoverableParser<I, Arc<Directive<T>>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::recursive(|| {
        punctuator("@")
            .and(name().recover(Missing::unary(Diagnostic::missing_directive_name)))
            .and(opt(arguments().map(Into::into)))
            .unzip()
            .map(|(at, name, arguments)| Directive {
                at,
                name,
                arguments,
            })
            .map(Into::into)
    })
}

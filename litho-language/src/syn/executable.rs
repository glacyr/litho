use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use nom::IResult;
use wrom::{alt, delimited, many0, many1, opt, recursive, Input, RecoverableParser};
use wrom_derive::wrom;

use crate::ast::*;
use crate::lex::{Token, TokenKind};

use super::combinators::{
    float_value, int_value, keyword, name, name_unless, punctuator, string_value,
};
use super::{recovery::RecoveryPoint, Error, RECURSION_LIMIT};

#[wrom]
pub fn executable_document<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDocument<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many0(executable_definition()).map(|definitions| ExecutableDocument { definitions })
}

#[wrom]
pub fn executable_definition<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDefinition<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        operation_definition()
            .map(Into::into)
            .map(ExecutableDefinition::OperationDefinition),
        fragment_definition()
            .map(Into::into)
            .map(ExecutableDefinition::FragmentDefinition),
    ))
}

#[wrom]
pub fn operation_definition<'a, T, I>(
) -> impl RecoverableParser<I, OperationDefinition<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        operation_type(),
        opt(name()),
        opt(variable_definitions()),
        opt(directives()),
        selection_set(RECURSION_LIMIT)
            .map(Into::into)
            .recover(Missing::unary(
                Diagnostic::missing_operation_definition_selection_set,
            )),
    )
        .map(
            |(ty, name, variable_definitions, directives, selection_set)| OperationDefinition {
                ty: Some(ty),
                name,
                variable_definitions,
                directives,
                selection_set,
            },
        )
}

#[wrom]
pub fn operation_type<'a, T, I>(
) -> impl RecoverableParser<I, OperationType<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        keyword(TokenKind::KeywordQuery).map(OperationType::Query),
        keyword(TokenKind::KeywordMutation).map(OperationType::Mutation),
        keyword(TokenKind::KeywordSubscription).map(OperationType::Subscription),
    ))
}

#[wrom]
pub fn selection_set<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, SelectionSet<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many0(recursive(depth, selection)),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_selection_set_closing_brace),
    )
    .map(|(brace_left, selections, brace_right)| SelectionSet {
        braces: (brace_left, brace_right),
        selections,
    })
}

#[wrom]
pub fn selection<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, Selection<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        fragment_spread().map(Selection::FragmentSpread),
        recursive(depth, inline_fragment).map(Selection::InlineFragment),
        recursive(depth, field)
            .map(Into::into)
            .map(Selection::Field),
    ))
}

#[wrom]
pub fn field<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, Field<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        name(),
        opt(punctuator(TokenKind::Colon)
            .and(name().recover(Missing::unary(Diagnostic::missing_field_name)))),
        opt(arguments().map(Into::into)),
        opt(directives()),
        opt(selection_set(depth).map(Into::into)),
    )
        .map(
            |(name, alias, arguments, directives, selection_set)| match alias {
                Some((colon, orig)) => Field {
                    alias: Some(Alias { name, colon }),
                    name: orig,
                    arguments,
                    directives,
                    selection_set,
                },
                None => Field {
                    alias: None,
                    name: name.into(),
                    arguments,
                    directives,
                    selection_set,
                },
            },
        )
}

#[wrom]
pub fn arguments<'a, T, I>() -> impl RecoverableParser<I, Arguments<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator(TokenKind::ParenLeft),
        many0(argument()),
        punctuator(TokenKind::ParenRight),
        Missing::binary(Diagnostic::missing_arguments_closing_parentheses),
    )
    .map(|(left, items, right)| Arguments {
        parens: (left, right),
        items,
    })
}

#[wrom]
pub fn argument<'a, T, I>() -> impl RecoverableParser<I, Arc<Argument<T>>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        name(),
        punctuator(TokenKind::Colon).recover(Missing::unary(Diagnostic::missing_argument_colon)),
        value(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_argument_value)),
    )
        .map(|(name, colon, value)| Argument { name, colon, value })
        .map(Into::into)
}

#[wrom]
pub fn fragment_spread<'a, T, I>(
) -> impl RecoverableParser<I, Arc<FragmentSpread<T>>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        punctuator(TokenKind::Dots),
        name_unless(TokenKind::KeywordOn),
        opt(directives()),
    )
        .map(|(dots, fragment_name, directives)| FragmentSpread {
            dots,
            fragment_name,
            directives,
        })
        .map(Into::into)
}

#[wrom]
pub fn inline_fragment<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, InlineFragment<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        punctuator(TokenKind::Dots),
        opt(type_condition()),
        opt(directives()),
        recursive(depth, selection_set)
            .map(Into::into)
            .recover(Missing::unary(
                Diagnostic::missing_inline_fragment_selection_set,
            )),
    )
        .map(
            |(dots, type_condition, directives, selection_set)| InlineFragment {
                dots,
                type_condition,
                directives,
                selection_set,
            },
        )
}

#[wrom]
pub fn fragment_definition<'a, T, I>(
) -> impl RecoverableParser<I, FragmentDefinition<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        keyword(TokenKind::KeywordFragment),
        name_unless(TokenKind::KeywordOn)
            .recover(Missing::unary(Diagnostic::missing_fragment_name)),
        type_condition().recover(Missing::unary(Diagnostic::missing_fragment_type_condition)),
        opt(directives()),
        selection_set(RECURSION_LIMIT)
            .map(Into::into)
            .recover(Missing::Unary(Diagnostic::missing_fragment_selection_set)),
    )
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
}

#[wrom]
pub fn type_condition<'a, T, I>(
) -> impl RecoverableParser<I, TypeCondition<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    keyword(TokenKind::KeywordOn)
        .and(named_type().recover(Missing::unary(
            Diagnostic::missing_type_condition_named_type,
        )))
        .map(|(on, named_type)| TypeCondition { on, named_type })
}

#[wrom]
pub fn value<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, Arc<Value<T>>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        int_value().map(Value::IntValue),
        float_value().map(Value::FloatValue),
        string_value().map(Value::StringValue),
        boolean_value().map(Value::BooleanValue),
        null_value().map(Value::NullValue),
        enum_value().map(Value::EnumValue),
        variable().map(Value::Variable),
        recursive(depth, list_value).map(Value::ListValue),
        recursive(depth, object_value).map(Value::ObjectValue),
    ))
    .map(Into::into)
}

#[wrom]
pub fn boolean_value<'a, T, I>(
) -> impl RecoverableParser<I, BooleanValue<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        keyword(TokenKind::KeywordTrue).map(BooleanValue::True),
        keyword(TokenKind::KeywordFalse).map(BooleanValue::False),
    ))
}

#[wrom]
pub fn null_value<'a, T, I>() -> impl RecoverableParser<I, NullValue<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    keyword(TokenKind::KeywordNull).map(NullValue)
}

#[wrom]
pub fn enum_value<'a, T, I>() -> impl RecoverableParser<I, EnumValue<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    name().map(EnumValue)
}

#[wrom]
pub fn list_value<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, ListValue<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator(TokenKind::BracketLeft),
        many0(value(depth)),
        punctuator(TokenKind::BracketRight),
        Missing::binary(Diagnostic::missing_list_value_closing_bracket),
    )
    .map(|(left, values, right)| ListValue {
        brackets: (left, right),
        values,
    })
}

#[wrom]
pub fn object_value<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, ObjectValue<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many0(object_field(depth)),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_object_value_closing_brace),
    )
    .map(|(left, object_fields, right)| ObjectValue {
        braces: (left, right),
        object_fields,
    })
}

#[wrom]
pub fn object_field<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, ObjectField<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        name(),
        punctuator(TokenKind::Colon)
            .recover(Missing::unary(Diagnostic::missing_object_field_colon)),
        value(depth).recover(Missing::unary(Diagnostic::missing_object_field_value)),
    )
        .map(|(name, colon, value)| ObjectField { name, colon, value })
}

#[wrom]
pub fn variable_definitions<'a, T, I>(
) -> impl RecoverableParser<I, VariableDefinitions<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator(TokenKind::ParenLeft),
        many0(variable_definition()),
        punctuator(TokenKind::ParenRight),
        Missing::binary(Diagnostic::missing_variable_definitions_closing_parenthesis),
    )
    .map(|(left, variable_definitions, right)| VariableDefinitions {
        parens: (left, right),
        variable_definitions,
    })
}

#[wrom]
pub fn variable_definition<'a, T, I>(
) -> impl RecoverableParser<I, Arc<VariableDefinition<T>>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        variable(),
        punctuator(TokenKind::Colon).recover(Missing::unary(
            Diagnostic::missing_variable_definition_colon,
        )),
        ty(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_variable_definition_type)),
        opt(default_value()),
        opt(directives()),
    )
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
}

#[wrom]
pub fn variable<'a, T, I>() -> impl RecoverableParser<I, Variable<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    punctuator(TokenKind::Dollar)
        .and(name().recover(Missing::unary(Diagnostic::missing_variable_name)))
        .map(|(dollar, name)| Variable { dollar, name })
}

#[wrom]
pub fn default_value<'a, T, I>(
) -> impl RecoverableParser<I, DefaultValue<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    punctuator(TokenKind::Eq)
        .and(value(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_default_value)))
        .map(|(eq, value)| DefaultValue { eq, value })
}

#[wrom]
pub fn ty<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, Arc<Type<T>>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        named_type().map(Type::Named),
        recursive(depth, list_type).map(Type::List),
    ))
    .and(opt(punctuator(TokenKind::Bang)))
    .map(|(ty, bang): (Type<_>, Option<_>)| match bang {
        Some(bang) => Type::NonNull(NonNullType {
            ty: ty.into(),
            bang,
        }),
        None => ty,
    })
    .map(Into::into)
}

#[wrom]
pub fn named_type<'a, T, I>() -> impl RecoverableParser<I, NamedType<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    name().map(NamedType)
}

#[wrom]
pub fn list_type<'a, T, I>(
    depth: usize,
) -> impl RecoverableParser<I, ListType<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    delimited(
        punctuator(TokenKind::BracketLeft),
        recursive(depth, ty).recover(Missing::unary(Diagnostic::missing_list_type_wrapped_type)),
        punctuator(TokenKind::BracketRight),
        Missing::binary(Diagnostic::missing_list_type_closing_bracket),
    )
    .map(|(left, ty, right)| ListType {
        brackets: (left, right),
        ty,
    })
}

#[wrom]
pub fn directives<'a, T, I>() -> impl RecoverableParser<I, Directives<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many1(directive()).map(|directives| Directives { directives })
}

#[wrom]
pub fn directive<'a, T, I>(
) -> impl RecoverableParser<I, Arc<Directive<T>>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    (
        punctuator(TokenKind::At),
        name().recover(Missing::unary(Diagnostic::missing_directive_name)),
        opt(arguments().map(Into::into)),
    )
        .map(|(at, name, arguments)| Directive {
            at,
            name,
            arguments,
        })
        .map(Into::into)
}

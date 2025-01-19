use litho_diagnostics::Diagnostic;
use wrom::{alt, delimited, opt, recursive, Input, Recoverable, RecoverableParser};
use wrom_derive::wrom;

use crate::ast::*;
use crate::lex::{Token, TokenKind};

use super::combinators::{
    float_value, int_value, keyword, name, name_unless_on, punctuator, string_value,
};
use super::{many_ext, Error, RecoverableParserExt, RecoveryPoint, RECURSION_LIMIT};

#[wrom]
pub fn executable_document<'a, T, I>() -> impl RecoverableParser<I, ExecutableDocument<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    many_ext(executable_definition()).map(|definitions| ExecutableDocument { definitions })
}

#[wrom]
pub fn executable_definition<'a, T, I>(
) -> impl RecoverableParser<I, ExecutableDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        operation_definition()
            .into_shared()
            .map(ExecutableDefinition::OperationDefinition),
        fragment_definition()
            .into_shared()
            .map(ExecutableDefinition::FragmentDefinition),
        selection_set(RECURSION_LIMIT)
            .into_shared()
            .map(|selection_set| OperationDefinition {
                ty: None,
                name: None,
                variable_definitions: None,
                directives: None,
                selection_set: selection_set.into(),
            })
            .into_shared()
            .map(ExecutableDefinition::OperationDefinition),
    ))
}

#[wrom]
pub fn operation_definition<'a, T, I>(
) -> impl RecoverableParser<I, OperationDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        operation_type(),
        opt(name()),
        opt(variable_definitions()),
        opt(directives()),
        selection_set(RECURSION_LIMIT)
            .into_shared()
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
pub fn operation_type<'a, T, I>() -> impl RecoverableParser<I, OperationType<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
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
) -> impl RecoverableParser<I, SelectionSet<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many_ext(recursive(depth, selection).ok_or_else(Error::max_recursion)),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_selection_set_closing_brace),
    )
    .map(|(brace_left, selections, brace_right)| SelectionSet {
        braces: (brace_left, brace_right),
        selections,
    })
}

#[wrom]
pub fn selection<'a, T, I>(depth: usize) -> impl RecoverableParser<I, Selection<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        field(depth).into_shared().map(Selection::Field),
        punctuator(TokenKind::Dots).flat_map(move |dots| selection_fragment_with_dots(dots, depth)),
    ))
}

#[wrom]
pub fn selection_fragment_with_dots<'a, T, I>(
    dots: Punctuator<'a, T>,
    depth: usize,
) -> impl RecoverableParser<I, Selection<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        inline_fragment(dots.clone(), depth).map(Selection::InlineFragment),
        fragment_spread(dots.clone()).map(Selection::FragmentSpread),
    ))
    .recover(Missing::unary(
        Diagnostic::missing_inline_fragment_selection_set,
    ))
    .map(move |fragment| match fragment {
        Recoverable::Present(fragment) => fragment,
        Recoverable::Missing(missing) => Selection::InlineFragment(InlineFragment {
            dots: dots.clone(),
            directives: None,
            type_condition: None,
            selection_set: Recoverable::Missing(missing),
        }),
    })
}

#[wrom]
pub fn field<'a, T, I>(depth: usize) -> impl RecoverableParser<I, Field<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        name(),
        opt(punctuator(TokenKind::Colon)
            .and(name().recover(Missing::unary(Diagnostic::missing_field_name)))),
        opt(arguments().into_shared()),
        opt(directives()),
        opt(selection_set(depth).into_shared()),
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
pub fn arguments<'a, T, I>() -> impl RecoverableParser<I, Arguments<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::ParenLeft),
        many_ext(argument()),
        punctuator(TokenKind::ParenRight),
        Missing::binary(Diagnostic::missing_arguments_closing_parentheses),
    )
    .map(|(left, items, right)| Arguments {
        parens: (left, right),
        items,
    })
}

#[wrom]
pub fn argument<'a, T, I>() -> impl RecoverableParser<I, Shared<'a, T, Argument<'a, T>>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        name(),
        punctuator(TokenKind::Colon).recover(Missing::unary(Diagnostic::missing_argument_colon)),
        value(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_argument_value)),
    )
        .map(|(name, colon, value)| Argument { name, colon, value })
        .into_shared()
}

#[wrom]
pub fn fragment_spread<'a, T, I>(
    dots: Punctuator<'a, T>,
) -> impl RecoverableParser<I, Shared<'a, T, FragmentSpread<'a, T>>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    name_unless_on()
        .and(opt(directives()))
        .map(move |(fragment_name, directives)| FragmentSpread {
            dots: dots.clone(),
            fragment_name,
            directives,
        })
        .into_shared()
}

#[wrom]
pub fn inline_fragment<'a, T, I>(
    dots: Punctuator<'a, T>,
    depth: usize,
) -> impl RecoverableParser<I, InlineFragment<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    opt(type_condition())
        .and_recognize(opt(directives()))
        .and_recognize(selection_set(depth).into_shared().recover(Missing::unary(
            Diagnostic::missing_inline_fragment_selection_set,
        )))
        .map(
            move |((type_condition, directives), selection_set)| InlineFragment {
                dots: dots.clone(),
                type_condition,
                directives,
                selection_set,
            },
        )
}

#[wrom]
pub fn fragment_definition<'a, T, I>() -> impl RecoverableParser<I, FragmentDefinition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        keyword(TokenKind::KeywordFragment),
        name_unless_on().recover(Missing::unary(Diagnostic::missing_fragment_name)),
        type_condition().recover(Missing::unary(Diagnostic::missing_fragment_type_condition)),
        opt(directives()),
        selection_set(RECURSION_LIMIT)
            .into_shared()
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
pub fn type_condition<'a, T, I>() -> impl RecoverableParser<I, TypeCondition<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
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
) -> impl RecoverableParser<I, Shared<'a, T, Value<'a, T>>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        int_value().map(Value::IntValue),
        float_value().map(Value::FloatValue),
        string_value().map(Value::StringValue),
        boolean_value().map(Value::BooleanValue),
        null_value().map(Value::NullValue),
        enum_value().map(Value::EnumValue),
        variable().map(Value::Variable),
        list_value(depth).map(Value::ListValue),
        object_value(depth).map(Value::ObjectValue),
    ))
    .into_shared()
}

#[wrom]
pub fn boolean_value<'a, T, I>() -> impl RecoverableParser<I, BooleanValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        keyword(TokenKind::KeywordTrue).map(BooleanValue::True),
        keyword(TokenKind::KeywordFalse).map(BooleanValue::False),
    ))
}

#[wrom]
pub fn null_value<'a, T, I>() -> impl RecoverableParser<I, NullValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    keyword(TokenKind::KeywordNull).map(NullValue)
}

#[wrom]
pub fn enum_value<'a, T, I>() -> impl RecoverableParser<I, EnumValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    name().map(EnumValue)
}

#[wrom]
pub fn list_value<'a, T, I>(depth: usize) -> impl RecoverableParser<I, ListValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BracketLeft),
        many_ext(recursive(depth, value).ok_or_else(Error::max_recursion)),
        punctuator(TokenKind::BracketRight),
        Missing::binary(Diagnostic::missing_list_value_closing_bracket),
    )
    .map(|(left, values, right)| ListValue {
        brackets: (left, right),
        values,
    })
}

#[wrom]
pub fn object_value<'a, T, I>(depth: usize) -> impl RecoverableParser<I, ObjectValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::BraceLeft),
        many_ext(object_field(depth)),
        punctuator(TokenKind::BraceRight),
        Missing::binary(Diagnostic::missing_object_value_closing_brace),
    )
    .map(|(left, object_fields, right)| ObjectValue {
        braces: (left, right),
        object_fields,
    })
}

#[wrom]
pub fn object_field<'a, T, I>(depth: usize) -> impl RecoverableParser<I, ObjectField<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        name(),
        punctuator(TokenKind::Colon)
            .recover(Missing::unary(Diagnostic::missing_object_field_colon)),
        recursive(depth, value)
            .ok_or_else(Error::max_recursion)
            .recover(Missing::unary(Diagnostic::missing_object_field_value)),
    )
        .map(|(name, colon, value)| ObjectField { name, colon, value })
}

#[wrom]
pub fn variable_definitions<'a, T, I>(
) -> impl RecoverableParser<I, VariableDefinitions<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        punctuator(TokenKind::ParenLeft),
        many_ext(variable_definition()),
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
) -> impl RecoverableParser<I, Shared<'a, T, VariableDefinition<'a, T>>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
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
        .into_shared()
}

#[wrom]
pub fn variable<'a, T, I>() -> impl RecoverableParser<I, Variable<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    punctuator(TokenKind::Dollar)
        .and(name().recover(Missing::unary(Diagnostic::missing_variable_name)))
        .map(|(dollar, name)| Variable { dollar, name })
}

#[wrom]
pub fn default_value<'a, T, I>() -> impl RecoverableParser<I, DefaultValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    punctuator(TokenKind::Eq)
        .and(value(RECURSION_LIMIT).recover(Missing::unary(Diagnostic::missing_default_value)))
        .map(|(eq, value)| DefaultValue { eq, value })
}

#[wrom]
pub fn ty<'a, T, I>(depth: usize) -> impl RecoverableParser<I, Shared<'a, T, Type<'a, T>>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    alt((
        named_type().map(Type::Named),
        recursive(depth, list_type::<T, I>)
            .ok_or_else(Error::max_recursion)
            .map(Type::List),
    ))
    .and(opt(punctuator(TokenKind::Bang)))
    .map_input(|input: &mut I, (ty, bang)| match bang {
        Some(bang) => Type::NonNull(NonNullType {
            ty: input.shared(ty),
            bang,
        }),
        None => ty,
    })
    .into_shared()
}

#[wrom]
pub fn named_type<'a, T, I>() -> impl RecoverableParser<I, NamedType<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    name().map(NamedType)
}

#[wrom]
pub fn list_type<'a, T, I>(depth: usize) -> impl RecoverableParser<I, ListType<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    delimited(
        RecoveryPoint::from(TokenKind::BracketLeft),
        ty(depth).recover(Missing::unary(Diagnostic::missing_list_type_wrapped_type)),
        punctuator(TokenKind::BracketRight),
        Missing::binary(Diagnostic::missing_list_type_closing_bracket),
    )
    .map(|(left, ty, right)| ListType {
        brackets: (left, right),
        ty,
    })
}

#[wrom]
pub fn directives<'a, T, I>() -> impl RecoverableParser<I, Directives<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    many_ext(directive()).map(|directives| Directives { directives })
}

#[wrom]
pub fn directive<'a, T, I>() -> impl RecoverableParser<I, Shared<'a, T, Directive<'a, T>>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>> + Context<'a, T> + Spanned,
    T: ContextValue<'a> + 'a,
{
    (
        punctuator(TokenKind::At),
        name().recover(Missing::unary(Diagnostic::missing_directive_name)),
        opt(arguments().into_shared()),
    )
        .map(|(at, name, arguments)| Directive {
            at,
            name,
            arguments,
        })
        .into_shared()
}

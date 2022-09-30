use wrom::branch::alt;
use wrom::combinator::opt;
use wrom::multi::many0;
use wrom::sequence::delimited;
use wrom::{Input, Recognizer, RecoverableParser};

use crate::ast::*;
use crate::lex::{Name, Token};

use super::combinators::{keyword, name, punctuator};
use super::Error;

pub fn executable_definition<'a, I>(
) -> impl RecoverableParser<I, ExecutableDefinition<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    alt((
        operation_definition().map(ExecutableDefinition::OperationDefinition),
        fragment_definition().map(ExecutableDefinition::FragmentDefinition),
    ))
}

pub fn operation_definition<'a, I>() -> impl RecoverableParser<I, OperationDefinition<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    operation_type()
        .and(name().recover(|| "Missing name of this operation definition.".into()))
        .and(opt(variable_definitions()))
        .and(opt(directives()))
        .and(selection_set().recover(|| "Operation is missing selection set.".into()))
        .flatten()
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

pub fn operation_type<'a, I>() -> impl RecoverableParser<I, OperationType<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    alt((
        keyword("query").map(OperationType::Query),
        keyword("mutation").map(OperationType::Mutation),
        keyword("subscription").map(OperationType::Subscription),
    ))
}

pub fn selection_set<'a, I>() -> impl RecoverableParser<I, SelectionSet<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    wrom::recoverable_parser(
        |input, recovery_point: &'_ dyn Recognizer<I, Error<'a>>| {
            delimited(
                punctuator("{"),
                many0(selection()),
                punctuator("}"),
                Missing::delimiter_complaint(
                    "Unlimited selection set.",
                    "This `{` here ...",
                    "... should have a corresponding `}` here.",
                ),
            )
            .map(|(brace_left, selections, brace_right)| SelectionSet {
                braces: (brace_left, brace_right),
                selections,
            })
            .parse(input, recovery_point)
        },
        move |input| punctuator("{").recognize(input),
    )
}

pub fn selection<'a, I>() -> impl RecoverableParser<I, Selection<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    alt((
        inline_fragment().map(Selection::InlineFragment),
        fragment_spread().map(Selection::FragmentSpread),
        field().map(Selection::Field),
    ))
}

fn field<'a, I>() -> impl RecoverableParser<I, Field<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    alt((
        alias()
            .and(name().recover(|| "Field should have a name.".into()))
            .and(opt(arguments()))
            .and(opt(directives()))
            .and(opt(selection_set()))
            .flatten()
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
            .and(opt(arguments()))
            .and(opt(directives()))
            .and(opt(selection_set()))
            .flatten()
            .map(
                |(name, arguments, directives, selection_set): (Name<'a>, _, _, _)| Field {
                    alias: None,
                    name: name.into(),
                    arguments,
                    directives,
                    selection_set,
                },
            ),
    ))
}

pub fn alias<'a, I>() -> impl RecoverableParser<I, Alias<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    name()
        .and(punctuator(":"))
        .map(|(name, colon)| Alias { name, colon })
}

pub fn arguments<'a, I>() -> impl RecoverableParser<I, Arguments<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    delimited(
        punctuator("("),
        many0(argument()),
        punctuator(")"),
        Missing::delimiter_complaint(
            "Arguments are missing closing parenthesis.",
            "This `(` here ...",
            "... should have a corresponding `)` here.",
        ),
    )
    .map(|(left, items, right)| Arguments {
        parens: (left, right),
        items,
    })
}

pub fn argument<'a, I>() -> impl RecoverableParser<I, Argument<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    name()
        .and(punctuator(":").recover(|| "Missing colon here.".into()))
        .and(value().recover(|| "Missing value here.".into()))
        .flatten()
        .map(|(name, colon, value)| Argument { name, colon, value })
}

pub fn fragment_spread<'a, I>() -> impl RecoverableParser<I, FragmentSpread<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    punctuator("...")
        .and(name().recover(|| "Fragment spread is missing name here.".into()))
        .and(opt(directives()))
        .flatten()
        .map(|(dots, fragment_name, directives)| FragmentSpread {
            dots,
            fragment_name,
            directives,
        })
}

pub fn inline_fragment<'a, I>() -> impl RecoverableParser<I, InlineFragment<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    punctuator("...")
        .and(opt(type_condition()))
        .and(opt(directives()))
        .and(selection_set().recover(|| "Inline fragment is missing selection set.".into()))
        .flatten()
        .map(
            |(dots, type_condition, directives, selection_set)| InlineFragment {
                dots,
                type_condition,
                directives,
                selection_set,
            },
        )
}

pub fn fragment_definition<'a, I>() -> impl RecoverableParser<I, FragmentDefinition<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    keyword("fragment")
        .and(name().recover(|| "Fragment definition is missing a name.".into()))
        .and(type_condition().recover(|| "Fragment definition is missing a type condition.".into()))
        .and(opt(directives()))
        .and(selection_set().recover(|| "Fragment definition is missing a selection set.".into()))
        .flatten()
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

pub fn type_condition<'a, I>() -> impl RecoverableParser<I, TypeCondition<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    keyword("on")
        .and(name().recover(|| "Type condition is missing name of type.".into()))
        .map(|(on, named_type)| TypeCondition { on, named_type })
}

pub fn value<'a, I>() -> impl RecoverableParser<I, Value<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    alt((
        boolean_value().map(Value::BooleanValue),
        null_value().map(Value::NullValue),
    ))
}

// impl<'a> Parse<'a> for Value<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Input<Item = Token<'a>, Missing = Missing>,
//     {
//         alt((
//             // IntValue::parse.map(Value::IntValue),
//             // FloatValue::parse.map(Value::FloatValue),
//             BooleanValue::parse.map(Value::BooleanValue),
//             NullValue::parse.map(Value::NullValue),
//             EnumValue::parse.map(Value::EnumValue),
//             Variable::parse.map(Value::Variable),
//             ListValue::parse.map(Value::ListValue),
//             ObjectValue::parse.map(Value::ObjectValue),
//         ))
//         .parse(input)
//     }
// }

pub fn boolean_value<'a, I>() -> impl RecoverableParser<I, BooleanValue<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    alt((
        keyword("true").map(BooleanValue::True),
        keyword("false").map(BooleanValue::False),
    ))
}

pub fn null_value<'a, I>() -> impl RecoverableParser<I, NullValue<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    keyword("null").map(NullValue)
}

// impl<'a> Parse<'a> for EnumValue<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Input<Item = Token<'a>, Missing = Missing>,
//     {
//         name_if_fn(|name| match name {
//             "true" | "false" | "null" => false,
//             _ => true,
//         })
//         .map(EnumValue)
//         .parse(input)
//     }
// }

// impl<'a> Parse<'a> for ListValue<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Input<Item = Token<'a>, Missing = Missing>,
//     {
//         group::<Value, I>("[", "]")
//             .map(|(left, values, right)| ListValue {
//                 brackets: (left, right),
//                 values,
//             })
//             .parse(input)
//     }
// }

// impl<'a> Parse<'a> for ObjectValue<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Input<Item = Token<'a>, Missing = Missing>,
//     {
//         group::<ObjectField, I>("{", "}")
//             .map(|(left, object_fields, right)| ObjectValue {
//                 braces: (left, right),
//                 object_fields,
//             })
//             .parse(input)
//     }
// }

// impl<'a> Parse<'a> for ObjectField<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Input<Item = Token<'a>, Missing = Missing>,
//     {
//         tuple((Name::parse, punctuator_if(":"), Value::parse))
//             .map(|(name, colon, value)| ObjectField { name, colon, value })
//             .parse(input)
//     }
// }

fn variable_definitions<'a, I>() -> impl RecoverableParser<I, VariableDefinitions<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    punctuator("(")
        .and(many0(variable_definition()))
        .and_recover(punctuator(")"), |(left, _)| {
            Missing::Delimiter(
                "Undelimited variable definitions.",
                "Expected this ( here ...",
                left.span(),
                "... to match a ) here.",
            )
        })
        .flatten()
        .map(|(left, variable_definitions, right)| VariableDefinitions {
            parens: (left, right),
            variable_definitions,
        })
}

fn variable_definition<'a, I>() -> impl RecoverableParser<I, VariableDefinition<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    variable()
        .and(punctuator(":").recover(|| "Expected a `:` here.".into()))
        .and(ty().recover(|| "Expected a type here.".into()))
        .and(opt(default_value()))
        .and(opt(directives()))
        .flatten()
        .map(
            |(variable, colon, ty, default_value, directives)| VariableDefinition {
                variable,
                colon,
                ty,
                default_value,
                directives,
            },
        )
}

pub fn variable<'a, I>() -> impl RecoverableParser<I, Variable<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    punctuator("$")
        .and(name())
        .map(|(dollar, name)| Variable { dollar, name })
}

pub fn default_value<'a, I>() -> impl RecoverableParser<I, DefaultValue<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    punctuator("=")
        .and(value().recover(|| "Expected a value here.".into()))
        .map(|(eq, value)| DefaultValue { eq, value })
}

pub fn ty<'a, I>() -> impl RecoverableParser<I, Type<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    alt((
        non_null_type().map(Box::new).map(Type::NonNull),
        named_type().map(Type::Named),
        list_type().map(Box::new).map(Type::List),
    ))
}

pub fn named_type<'a, I>() -> impl RecoverableParser<I, NamedType<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    name().map(NamedType)
}

pub fn list_type<'a, I>() -> impl RecoverableParser<I, ListType<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    wrom::recoverable_parser(
        |input, recovery_point: &'_ dyn Recognizer<I, Error<'a>>| {
            delimited(
                punctuator("["),
                ty().recover(|| "Expected a type here.".into()),
                punctuator("]"),
                Missing::delimiter_complaint(
                    "List type is missing closing delimiter.",
                    "This `[` here ...",
                    "... should have a corresponding `]` here.",
                ),
            )
            .map(|(left, ty, right)| ListType {
                brackets: (left, right),
                ty,
            })
            .parse(input, recovery_point)
        },
        |input| punctuator("[").recognize(input),
    )
}

pub fn non_null_type<'a, I>() -> impl RecoverableParser<I, NonNullType<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    alt((
        named_type().map(Type::Named),
        list_type().map(Box::new).map(Type::List),
    ))
    .and(punctuator("!"))
    .map(|(ty, bang)| NonNullType { ty, bang })
}

pub fn directives<'a, I>() -> impl RecoverableParser<I, Directives<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    many0(directive()).map(|directives| Directives { directives })
}

pub fn directive<'a, I>() -> impl RecoverableParser<I, Directive<'a>, Error<'a>>
where
    I: Input<Item = Token<'a>, Missing = Missing>,
{
    punctuator("@")
        .and(name().recover(|| "Expected the name of a directive here.".into()))
        .and(opt(arguments()))
        .flatten()
        .map(|(at, name, arguments)| Directive {
            at,
            name,
            arguments,
        })
}

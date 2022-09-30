use nom::InputLength;
use wrom::branch::alt;
use wrom::combinator::opt;
use wrom::multi::many0;
use wrom::sequence::tuple;
use wrom::{Recognizer, RecoverableParser};

use crate::ast::*;
use crate::lex::{Name, Token};

use super::combinators::{keyword, name, punctuator};
use super::Error;

// pub fn executable_definition<'a, I>() -> impl RecoverableParser<I, ExecutableDefinition<'a>,

// impl<'a> Parse<'a> for ExecutableDefinition<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         OperationDefinition::parse
//             .map(|definition| ExecutableDefinition::OperationDefinition(definition))
//             .parse(input)
//     }
// }

pub fn operation_definition<'a, I>() -> impl RecoverableParser<I, OperationDefinition<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((
        operation_type(),
        name(),
        opt(variable_definitions()),
        opt(directives()),
        selection_set(),
    ))
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
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    wrom::recoverable_parser(
        |input, recovery_point: &'_ dyn Recognizer<I, Error<'a>>| {
            tuple((punctuator("{"), many0(selection()), punctuator("}")))
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
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    // alt((
    // inline_fragment().map(Selection::InlineFragment),
    // fragment_spread().map(Selection::FragmentSpread),
    field().map(Selection::Field)
    // ))
}

// impl<'a> Parse<'a> for Selection<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         alt((
//             InlineFragment::parse.map(Selection::InlineFragment),
//             FragmentSpread::parse.map(Selection::FragmentSpread),
//             Field::parse.map(Selection::Field),
//         ))
//         .parse(input)
//     }
// }

fn field<'a, I>() -> impl RecoverableParser<I, Field<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    alt((
        tuple((
            alias(),
            name(),
            opt(arguments()),
            opt(directives()),
            opt(selection_set()),
        ))
        .map(
            |(alias, name, arguments, directives, selection_set)| Field {
                alias: Some(alias),
                name,
                arguments,
                directives,
                selection_set,
            },
        ),
        tuple((
            name(),
            opt(arguments()),
            opt(directives()),
            opt(selection_set()),
        ))
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
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    name()
        .and(punctuator(":"))
        .map(|(name, colon)| Alias { name, colon })
}

// impl<'a> Parse<'a> for Alias<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         Name::parse
//             .and(punctuator_if(":"))
//             .map(|(name, colon)| Alias { name, colon })
//             .parse(input)
//     }
// }

pub fn arguments<'a, I>() -> impl RecoverableParser<I, Arguments<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((punctuator("("), many0(argument()), punctuator(")"))).map(|(left, items, right)| {
        Arguments {
            parens: (left, right),
            items,
        }
    })
}

pub fn argument<'a, I>() -> impl RecoverableParser<I, Argument<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((name(), punctuator(":"), value())).map(|(name, colon, value)| Argument {
        name,
        colon,
        value,
    })
}

// impl<'a> Parse<'a> for FragmentSpread<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         tuple((
//             punctuator_if("..."),
//             rest,
//             name_if_fn(|name| name != "on"),
//             opt(Directives::parse),
//         ))
//         .map(|(dots, r1, fragment_name, directives)| FragmentSpread {
//             dots,
//             fragment_name,
//             directives,
//             rest: r1,
//         })
//         .parse(input)
//     }
// }

// impl<'a> Parse<'a> for InlineFragment<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         tuple((
//             punctuator_if("..."),
//             rest,
//             opt(TypeCondition::parse),
//             opt(Directives::parse),
//             SelectionSet::parse,
//         ))
//         .map(
//             |(dots, r1, type_condition, directives, selection_set)| InlineFragment {
//                 dots,
//                 type_condition,
//                 directives,
//                 selection_set,
//                 rest: r1,
//             },
//         )
//         .parse(input)
//     }
// }

// impl<'a> Parse<'a> for FragmentDefinition<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         tuple((
//             name_if("fragment"),
//             Name::parse,
//             TypeCondition::parse,
//             opt(Directives::parse),
//             SelectionSet::parse,
//         ))
//         .map(
//             |(fragment, fragment_name, type_condition, directives, selection_set)| {
//                 FragmentDefinition {
//                     fragment,
//                     fragment_name,
//                     type_condition,
//                     directives,
//                     selection_set,
//                 }
//             },
//         )
//         .parse(input)
//     }
// }

// impl<'a> Parse<'a> for TypeCondition<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         tuple((name_if("on"), Name::parse))
//             .map(|(on, named_type)| TypeCondition { on, named_type })
//             .parse(input)
//     }
// }

pub fn value<'a, I>() -> impl RecoverableParser<I, Value<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    alt((
        boolean_value().map(Value::BooleanValue),
        null_value().map(Value::NullValue),
    ))
}

// impl<'a> Parse<'a> for Value<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
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
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
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
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
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
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
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
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         tuple((Name::parse, punctuator_if(":"), Value::parse))
//             .map(|(name, colon, value)| ObjectField { name, colon, value })
//             .parse(input)
//     }
// }

fn variable_definitions<'a, I>() -> impl RecoverableParser<I, VariableDefinitions<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((
        punctuator("("),
        many0(variable_definition()),
        punctuator(")"),
    ))
    .map(|(left, variable_definitions, right)| VariableDefinitions {
        parens: (left, right),
        variable_definitions,
    })
}

// impl<'a> Parse<'a> for VariableDefinitions<'a> {
//     fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
//     where
//         I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     {
//         recoverable_group::<VariableDefinition, I>("(", ")")
//             .map(|(left, variable_definitions, right)| VariableDefinitions {
//                 parens: (left, right),
//                 variable_definitions,
//             })
//             .parse(input)
//     }
// }

fn variable_definition<'a, I>() -> impl RecoverableParser<I, VariableDefinition<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((
        variable(),
        punctuator(":"),
        ty(),
        opt(default_value()),
        opt(directives()),
    ))
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
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((punctuator("$"), name())).map(|(dollar, name)| Variable { dollar, name })
}

pub fn default_value<'a, I>() -> impl RecoverableParser<I, DefaultValue<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((punctuator("="), value())).map(|(eq, value)| DefaultValue { eq, value })
}

pub fn ty<'a, I>() -> impl RecoverableParser<I, Type<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    alt((
        non_null_type().map(Box::new).map(Type::NonNull),
        named_type().map(Type::Named),
        list_type().map(Box::new).map(Type::List),
    ))
}

pub fn named_type<'a, I>() -> impl RecoverableParser<I, NamedType<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    name().map(NamedType)
}

pub fn list_type<'a, I>() -> impl RecoverableParser<I, ListType<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    wrom::recoverable_parser(
        |input, recovery_point: &'_ dyn Recognizer<I, Error<'a>>| {
            tuple((punctuator("["), ty(), punctuator("]")))
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
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
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
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    many0(directive()).map(|directives| Directives { directives })
}

pub fn directive<'a, I>() -> impl RecoverableParser<I, Directive<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone + InputLength,
{
    tuple((punctuator("@"), name(), opt(arguments()))).map(|(at, name, arguments)| Directive {
        at,
        name,
        arguments,
    })
}

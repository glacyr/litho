use std::iter::once;

use nom::branch::alt;
use nom::combinator::{eof, fail, not, opt, peek};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many0, many1, many_till};
use nom::sequence::tuple;
use nom::{IResult, InputLength, Parser};
use nom_recovery::combinator::{concat_rest, recoverable_rest};
use nom_recovery::grammar;
use nom_recovery::sequence::recoverable_tuple;

use crate::ast::*;
use crate::lex::{lexer, Name, Punctuator, Span, Token};

use super::combinators::{
    group, name_if, name_if_fn, punctuator_if, recoverable_group, recoverable_opt,
    recoverable_punctuator_if, recovery_point, rest,
};
use super::{Error, Parse};

impl<'a> Parse<'a> for ExecutableDefinition<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        OperationDefinition::parse
            .map(|definition| ExecutableDefinition::OperationDefinition(definition))
            .parse(input)
    }
}

grammar! {
    Error = Error<'a>;

    OperationDefinition<'a>:
        | ty name variable_definitions directives selection_set;

    OperationType<'a>:
        | "query" => OperationType::Query
        | "mutation" => OperationType::Mutation
        | "subscription" => OperationType::Subscription;
}

impl<'a> Parse<'a> for OperationType<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        alt((
            name_if("query").map(|name| OperationType::Query(name)),
            name_if("mutation").map(|name| OperationType::Mutation(name)),
            name_if("subscription").map(|name| OperationType::Subscription(name)),
        ))(input)
    }
}

impl<'a> Parse<'a> for SelectionSet<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        recoverable_group::<Selection, I>("{", "}")
            .map(|(brace_left, selections, brace_right)| SelectionSet {
                braces: (brace_left, brace_right),
                selections,
            })
            .parse(input)
    }
}

impl<'a> Parse<'a> for Selection<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        alt((
            InlineFragment::parse.map(Selection::InlineFragment),
            FragmentSpread::parse.map(Selection::FragmentSpread),
            Field::parse.map(Selection::Field),
        ))
        .parse(input)
    }
}

impl<'a> Parse<'a> for Field<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((
            opt(Alias::parse),
            Name::parse,
            opt(Arguments::parse),
            opt(Directives::parse),
            opt(SelectionSet::parse),
        ))
        .map(
            |(alias, name, arguments, directives, selection_set)| Field {
                alias,
                name,
                arguments,
                directives,
                selection_set,
                rest: Rest(vec![]),
            },
        )
        .parse(input)
    }
}

impl<'a> Parse<'a> for Alias<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        Name::parse
            .and(punctuator_if(":"))
            .map(|(name, colon)| Alias { name, colon })
            .parse(input)
    }
}

impl<'a> Parse<'a> for Arguments<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        recoverable_group::<Argument<'a>, I>("(", ")")
            .map(|(left, items, right)| Arguments {
                parens: (left, right),
                items,
            })
            .parse(input)
    }
}

impl<'a> Parse<'a> for Argument<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((Name::parse, recoverable_punctuator_if(":"), Value::parse))
            .map(|(name, colon, value)| Argument { name, colon, value })
            .parse(input)
    }
}

impl<'a> Parse<'a> for FragmentSpread<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((
            punctuator_if("..."),
            rest,
            name_if_fn(|name| name != "on"),
            opt(Directives::parse),
        ))
        .map(|(dots, r1, fragment_name, directives)| FragmentSpread {
            dots,
            fragment_name,
            directives,
            rest: r1,
        })
        .parse(input)
    }
}

impl<'a> Parse<'a> for InlineFragment<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((
            punctuator_if("..."),
            rest,
            opt(TypeCondition::parse),
            opt(Directives::parse),
            SelectionSet::parse,
        ))
        .map(
            |(dots, r1, type_condition, directives, selection_set)| InlineFragment {
                dots,
                type_condition,
                directives,
                selection_set,
                rest: r1,
            },
        )
        .parse(input)
    }
}

impl<'a> Parse<'a> for FragmentDefinition<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((
            name_if("fragment"),
            Name::parse,
            TypeCondition::parse,
            opt(Directives::parse),
            SelectionSet::parse,
        ))
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
        .parse(input)
    }
}

impl<'a> Parse<'a> for TypeCondition<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((name_if("on"), Name::parse))
            .map(|(on, named_type)| TypeCondition { on, named_type })
            .parse(input)
    }
}

impl<'a> Parse<'a> for Value<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        alt((
            // IntValue::parse.map(Value::IntValue),
            // FloatValue::parse.map(Value::FloatValue),
            BooleanValue::parse.map(Value::BooleanValue),
            NullValue::parse.map(Value::NullValue),
            EnumValue::parse.map(Value::EnumValue),
            Variable::parse.map(Value::Variable),
            ListValue::parse.map(Value::ListValue),
            ObjectValue::parse.map(Value::ObjectValue),
        ))
        .parse(input)
    }
}

impl<'a> Parse<'a> for BooleanValue<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        alt((
            name_if("true").map(BooleanValue::True),
            name_if("false").map(BooleanValue::False),
        ))
        .parse(input)
    }
}

impl<'a> Parse<'a> for NullValue<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        name_if("null").map(NullValue).parse(input)
    }
}

impl<'a> Parse<'a> for EnumValue<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        name_if_fn(|name| match name {
            "true" | "false" | "null" => false,
            _ => true,
        })
        .map(EnumValue)
        .parse(input)
    }
}

impl<'a> Parse<'a> for ListValue<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        group::<Value, I>("[", "]")
            .map(|(left, values, right)| ListValue {
                brackets: (left, right),
                values,
            })
            .parse(input)
    }
}

impl<'a> Parse<'a> for ObjectValue<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        group::<ObjectField, I>("{", "}")
            .map(|(left, object_fields, right)| ObjectValue {
                braces: (left, right),
                object_fields,
            })
            .parse(input)
    }
}

impl<'a> Parse<'a> for ObjectField<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((Name::parse, punctuator_if(":"), Value::parse))
            .map(|(name, colon, value)| ObjectField { name, colon, value })
            .parse(input)
    }
}

impl<'a> Parse<'a> for VariableDefinitions<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        recoverable_group::<VariableDefinition, I>("(", ")")
            .map(|(left, variable_definitions, right)| VariableDefinitions {
                parens: (left, right),
                variable_definitions,
            })
            .parse(input)
    }
}

impl<'a> Parse<'a> for VariableDefinition<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        recoverable_tuple(
            (
                Variable::parse,
                punctuator_if(":"),
                Type::parse,
                opt(DefaultValue::parse),
                opt(Directives::parse),
            ),
            || recovery_point,
        )
        .map(
            |((variable, colon, ty, default_value, directives), rest)| VariableDefinition {
                variable,
                colon,
                ty,
                default_value,
                directives,
                rest,
            },
        )
        .parse(input)
    }
}

impl<'a> Parse<'a> for Variable<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        recoverable_tuple((recoverable_punctuator_if("$"), Name::parse), || {
            recovery_point
        })
        .map(|((dollar, name), rest)| Variable { dollar, name, rest })
        .parse(input)
    }
}

impl<'a> Parse<'a> for DefaultValue<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((
            punctuator_if("="),
            recoverable_rest(recovery_point),
            Value::parse,
        ))
        .map(|(eq, rest, value)| DefaultValue { eq, value, rest })
        .parse(input)
    }
}

impl<'a> Parse<'a> for Type<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        alt((
            NonNullType::parse.map(Box::new).map(Type::NonNull),
            NamedType::parse.map(Type::Named),
            ListType::parse.map(Box::new).map(Type::List),
        ))
        .parse(input)
    }
}

impl<'a> Parse<'a> for NamedType<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        Name::parse.map(NamedType).parse(input)
    }
}

impl<'a> Parse<'a> for ListType<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        tuple((
            punctuator_if("["),
            Type::parse,
            recoverable_punctuator_if("]"),
        ))
        .map(|(left, ty, right)| ListType {
            brackets: (left, right),
            ty,
        })
        .parse(input)
    }
}

impl<'a> Parse<'a> for NonNullType<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        alt((
            NamedType::parse.map(Type::Named),
            ListType::parse.map(Box::new).map(Type::List),
        ))
        .and(punctuator_if("!"))
        .map(|(ty, bang)| NonNullType { ty, bang })
        .parse(input)
    }
}

impl<'a> Parse<'a> for Directives<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        many1(Directive::parse)
            .map(|directives| Directives { directives })
            .parse(input)
    }
}

impl<'a> Parse<'a> for Directive<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        recoverable_tuple(
            (punctuator_if("@"), Name::parse, opt(Arguments::parse)),
            || recovery_point,
        )
        .map(|((at, name, arguments), rest)| Directive {
            at,
            name,
            arguments,
            rest,
        })
        .parse(input)
    }
}

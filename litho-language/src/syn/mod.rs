use std::iter::once;

use nom::combinator::opt;
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many0, many_till};
use nom::{IResult, InputLength, Parser};

use crate::ast::*;
use crate::lex::{lexer, Name, Punctuator, Span, Token};

mod combinators;
mod executable;

#[derive(Debug)]
pub enum Error<'a> {
    Nom(ErrorKind),
    Incomplete,
    ExpectedName(Span, Vec<&'a str>),
    ExpectedPunctuator(Span, Vec<&'a str>),
    Multiple(Vec<Error<'a>>),
}

impl<'a, I> ParseError<I> for Error<'a>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Error::Nom(kind)
    }

    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        Error::Multiple(vec![Self::from_error_kind(input, kind), other])
    }

    fn from_char(_input: I, _: char) -> Self {
        unreachable!()
    }

    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Error::Incomplete, Error::Incomplete) => Error::Incomplete,
            (Error::ExpectedName(lhs_span, lhs), Error::ExpectedName(rhs_span, rhs))
                if lhs_span == rhs_span =>
            {
                Error::ExpectedName(lhs_span, vec![lhs, rhs].concat())
            }
            (
                Error::ExpectedPunctuator(lhs_span, lhs),
                Error::ExpectedPunctuator(rhs_span, rhs),
            ) if lhs_span == rhs_span => {
                Error::ExpectedPunctuator(lhs_span, vec![lhs, rhs].concat())
            }
            (Error::Multiple(lhs), Error::Multiple(rhs)) => {
                Error::Multiple(lhs.into_iter().chain(rhs.into_iter()).collect())
            }
            (Error::Multiple(lhs), rhs) => {
                Error::Multiple(lhs.into_iter().chain(once(rhs)).collect())
            }
            (lhs, Error::Multiple(rhs)) => {
                Error::Multiple(once(lhs).into_iter().chain(rhs).collect())
            }
            (lhs, rhs) => Error::Multiple(vec![lhs, rhs]),
        }
    }
}

pub trait Parse<'a>: Sized {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength;

    fn parse_from_str(source_id: usize, input: &'a str) -> Result<Self, Error<'a>> {
        match Self::parse(lexer(source_id, input).exact()) {
            Ok((_, result)) => Ok(result),
            Err(nom::Err::Error(error) | nom::Err::Failure(error)) => Err(error),
            Err(nom::Err::Incomplete(_)) => Err(Error::Incomplete),
        }
    }
}

impl<'a, T> Parse<'a> for Option<T>
where
    T: Parse<'a>,
{
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        opt(T::parse).parse(input)
    }
}

impl<'a, T> Parse<'a> for Recoverable<'a, T>
where
    T: Parse<'a>,
{
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        opt(T::parse)
            .parse(input)
            .map(|(input, output)| (input, output.ok_or(vec![])))
    }
}

impl<'a> Parse<'a> for Token<'a> {
    fn parse<I>(mut input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        match input.next() {
            Some(token) => Ok((input, token)),
            None => Err(nom::Err::Error(Error::Incomplete)),
        }
    }
}

impl<'a> Parse<'a> for Name<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        let (input, token) = Token::parse(input)?;

        match token {
            Token::Name(name) => Ok((input, name)),
            token => Err(nom::Err::Error(Error::ExpectedName(token.span(), vec![]))),
        }
    }
}

impl<'a> Parse<'a> for Punctuator<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        let (input, token) = Token::parse(input)?;

        match token {
            Token::Punctuator(punctuator) => Ok((input, punctuator)),
            token => Err(nom::Err::Error(Error::ExpectedPunctuator(
                token.span(),
                vec![],
            ))),
        }
    }
}

impl<'a, T> Parse<'a> for Vec<Recoverable<'a, T>>
where
    T: Parse<'a>,
{
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        many0(
            many_till(Token::parse, T::parse).map(|(errors, item)| match errors.is_empty() {
                true => vec![Ok(item)],
                false => vec![Err(errors), Ok(item)],
            }),
        )
        .and(many0(Token::parse))
        .map(|(items, errors)| match errors.is_empty() {
            true => items.into_iter().flatten().collect(),
            false => items
                .into_iter()
                .flatten()
                .chain(once(Err(errors)))
                .collect(),
        })
        .parse(input)
    }
}

impl<'a> Parse<'a> for Document<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        Parse::parse
            .map(|definitions| Document { definitions })
            .parse(input)
    }
}

impl<'a> Parse<'a> for Definition<'a> {
    fn parse<I>(input: I) -> IResult<I, Self, Error<'a>>
    where
        I: Iterator<Item = Token<'a>> + Clone + InputLength,
    {
        ExecutableDefinition::parse
            .map(|definition| Definition::ExecutableDefinition(definition))
            .parse(input)
    }
}

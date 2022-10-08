use std::iter::once;

use nom::combinator::eof;
use nom::error::{ErrorKind, ParseError};
use nom::Err;
use wrom::branch::alt;
use wrom::multi::many0;
use wrom::{terminal, Input, RecoverableParser};

use crate::ast::*;
use crate::lex::{lexer, Token};

mod combinators;
pub mod executable;
mod parse;
pub mod schema;
mod stream;

pub use parse::Parse;
pub use stream::Stream;

#[derive(Debug)]
pub enum Error {
    Nom(ErrorKind),
    Incomplete,
    ExpectedKeyword(&'static str),
    ExpectedName,
    ExpectedPunctuator(&'static str),
    ExpectedIntValue,
    ExpectedFloatValue,
    ExpectedStringValue,
    Multiple(Vec<Error>),
}

impl<T, I> ParseError<I> for Error
where
    I: Iterator<Item = Token<T>> + Clone,
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
            (Error::ExpectedKeyword(lhs), Error::ExpectedKeyword(rhs)) if lhs == rhs => {
                Error::ExpectedKeyword(lhs)
            }
            (Error::ExpectedName, Error::ExpectedName) => Error::ExpectedName,
            (Error::ExpectedPunctuator(lhs), Error::ExpectedPunctuator(rhs)) if lhs == rhs => {
                Error::ExpectedPunctuator(lhs)
            }
            (Error::ExpectedIntValue, Error::ExpectedIntValue) => Error::ExpectedIntValue,
            (Error::ExpectedFloatValue, Error::ExpectedFloatValue) => Error::ExpectedFloatValue,
            (Error::ExpectedStringValue, Error::ExpectedStringValue) => Error::ExpectedStringValue,
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

pub fn document<'a, T, I>() -> impl RecoverableParser<I, Document<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many0(definition()).map(|definitions| Document { definitions })
}

pub fn definition<'a, T, I>() -> impl RecoverableParser<I, Definition<T>, Error> + 'a
where
    I: Input<Item = Token<T>, Missing = Missing> + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        executable::executable_definition().map(Definition::ExecutableDefinition),
        schema::type_system_definition_or_extension()
            .map(Definition::TypeSystemDefinitionOrExtension),
    ))
}

pub fn parse_from_str<'a, T, P, O>(
    parser: P,
    source_id: usize,
    input: &'a str,
) -> Result<(Vec<Token<T>>, O), Error>
where
    P: for<'b> RecoverableParser<Stream<'b, T>, O, Error>,
    T: From<&'a str> + Clone,
{
    match parser.parse((&lexer(source_id, input).exact()).into(), terminal(eof)) {
        Ok((input, result)) => Ok((input.into_unexpected(), result)),
        Err(nom::Err::Error(error) | nom::Err::Failure(error)) => Err(error),
        Err(nom::Err::Incomplete(_)) => Err(Error::Incomplete),
    }
}

macro_rules! parse {
    ($name:ident, $($fn:tt)*) => {
        impl<T> Parse<T> for $name<T> where T: for<'b> PartialEq<&'b str> + Clone {
            fn parse(stream: Stream<T>) -> Result<(Self, Vec<Token<T>>), Err<Error>> {
                $($fn)*()
                    .parse(stream, terminal(eof))
                    .map(|(input, value)| (value, input.into_unexpected()))
            }
        }
    };
}

parse!(Document, document);
// parse!(Definition, definition);
// parse!(ExecutableDocument, executable::executable_document);
// parse!(ExecutableDefinition, executable::executable_definition);
// parse!(OperationDefinition, executable::operation_definition);
// parse!(OperationType, executable::operation_type);
// parse!(SelectionSet, executable::selection_set);
// parse!(Selection, executable::selection);
// parse!(Field, executable::field);
// parse!(Alias, executable::alias);
// parse!(Arguments, executable::arguments);
// parse!(Argument, executable::argument);
// parse!(FragmentSpread, executable::fragment_spread);
// parse!(InlineFragment, executable::inline_fragment);
// parse!(FragmentDefinition, executable::fragment_definition);
// parse!(TypeCondition, executable::type_condition);
// parse!(Value, executable::value);
// parse!(BooleanValue, executable::boolean_value);
// parse!(NullValue, executable::null_value);
// parse!(EnumValue, executable::enum_value);
// parse!(ListValue, executable::list_value);
// parse!(ObjectValue, executable::object_value);
// parse!(VariableDefinitions, executable::variable_definitions);
// parse!(VariableDefinition, executable::variable_definition);
// parse!(Variable, executable::variable);
// parse!(Type, executable::ty);
// parse!(NamedType, executable::named_type);
// parse!(NonNullType, executable::non_null_type);
// parse!(Directives, executable::directives);
// parse!(Directive, executable::directive);

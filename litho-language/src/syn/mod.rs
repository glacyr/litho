use std::iter::once;
use std::sync::Arc;

use nom::combinator::eof;
use nom::error::{ErrorKind, ParseError};
use nom::{Err, Parser};
use wrom::{alt, many0, terminal, Input, RecoverableParser};

use crate::ast::*;
use crate::lex::Token;

mod combinators;
pub mod executable;
mod parse;
pub mod schema;
mod stream;

pub use parse::Parse;
pub use stream::Stream;

const RECURSION_LIMIT: usize = 64;

impl<I> wrom::Missing<I> for Missing
where
    I: Spanned,
{
    type Error = MissingToken;

    fn error(&self, input: &I) -> Self::Error {
        MissingToken {
            span: input.span(),
            missing: *self,
        }
    }
}

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

impl<I> ParseError<I> for Error {
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
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many0(definition().map(Into::into)).map(|definitions| Document { definitions })
}

pub fn definition<'a, T, I>() -> impl RecoverableParser<I, Definition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    alt((
        executable::executable_definition().map(Definition::ExecutableDefinition),
        schema::type_system_definition_or_extension()
            .map(Definition::TypeSystemDefinitionOrExtension),
    ))
}

macro_rules! parse {
    (Arc<$name:ident>, $($fn:tt)*) => {
        impl<T> Parse<T> for Arc<$name<T>> where T: for<'b> PartialEq<&'b str> + Clone {
            fn parse(stream: Stream<T>) -> Result<(Self, Vec<Token<T>>), Err<Error>> {
                $($fn)*
                    .parser(terminal(eof))
                    .parse(stream)
                    .map(|(input, value)| (value, input.into_unexpected()))
            }
        }
    };

    ($name:ident, $($fn:tt)*) => {
        impl<T> Parse<T> for $name<T> where T: for<'b> PartialEq<&'b str> + Clone {
            fn parse(stream: Stream<T>) -> Result<(Self, Vec<Token<T>>), Err<Error>> {
                $($fn)*
                    .parser(terminal(eof))
                    .parse(stream)
                    .map(|(input, value)| (value, input.into_unexpected()))
            }
        }
    };
}

parse!(Document, document());
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
parse!(Arc<Value>, executable::value(RECURSION_LIMIT));
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

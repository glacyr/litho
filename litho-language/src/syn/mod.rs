use std::iter::once;
use std::sync::Arc;

use nom::combinator::eof;
use nom::error::{ErrorKind, ParseError};
use nom::{Err, Parser};
use wrom::{alt, many0, terminal, Input, RecoverableParser};
use wrom_derive::wrom;

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

pub fn parser<I, T>(input: &mut I) -> Document<T>
where
    I: Iterator<Item = Token<T>>,
    T: for<'a> PartialEq<&'a str> + Clone,
{
    while let Some(token) = input.next() {
        match token {
            Token::Name(name) if name.as_ref() == &"type" => {
                parser_type(name, input);
            }
            _ => {
                // Unrecognized token
            }
        }
    }

    Document {
        definitions: vec![],
    }
}

pub fn parser_type<I, T>(ty: Name<T>, input: &mut I) -> ObjectTypeDefinition<T>
where
    I: Iterator<Item = Token<T>>,
    T: for<'a> PartialEq<&'a str> + Clone,
{
    let name = loop {
        match input.next() {
            Some(Token::Name(name)) => break name,
            Some(_) => {}
            None => todo!(),
        };
    };

    ObjectTypeDefinition {
        description: None,
        ty,
        name: name.into(),
        implements_interfaces: None,
        directives: None,
        fields_definition: None,
    }
}

pub fn document<'a, T, I>() -> impl RecoverableParser<I, Document<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    many0(definition().map(Into::into)).map(|definitions| Document { definitions })
}

#[wrom(schema::description().or(executable::executable_definition()).or(schema::type_system_definition_or_extension()))]
pub fn definition<'a, T, I>() -> impl RecoverableParser<I, Definition<T>, Error> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: for<'b> PartialEq<&'b str> + Clone + 'a,
{
    wrom::opt(schema::description())
        .and(alt((
            executable::executable_definition().map(Definition::ExecutableDefinition),
            schema::type_system_definition_or_extension()
                .map(Definition::TypeSystemDefinitionOrExtension),
        )))
        .map(|(_, b)| b)
}

macro_rules! parse {
    (Arc<$name:ident>, $($fn:tt)*) => {
        impl<T> Parse<T> for Arc<$name<T>> where T: for<'b> PartialEq<&'b str> + Clone {
            fn parse(stream: Stream<T>) -> Result<(Self, Vec<Token<T>>), Err<Error>> {
                $($fn)*
                    .parse(stream, terminal(eof))
                    .map(|(input, value)| (value, input.into_unexpected()))
            }
        }
    };

    ($name:ident, $($fn:tt)*) => {
        impl<T> Parse<T> for $name<T> where T: for<'b> PartialEq<&'b str> + Clone {
            fn parse(stream: Stream<T>) -> Result<(Self, Vec<Token<T>>), Err<Error>> {
                $($fn)*
                    .parse(stream, terminal(eof))
                    .map(|(input, value)| (value, input.into_unexpected()))
            }
        }
    };
}

parse!(Document, document());
// parse!(Definition, definition);
// parse!(ExecutableDocument, executable::executable_document);
// parse!(ExecutableDefinition, executable::executable_definition);
// parse!(OperationDefinition, executable::operation_definition());
// parse!(SchemaDefinition, schema::schema_definition());
// parse!(ExecutableDocument, executable::executable_document());
// parse!(TypeSystemDocument, schema::type_system_document());
// parse!(DirectiveDefinition, schema::directive_definition());
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
// parse!(Arc<Value>, executable::value(RECURSION_LIMIT));
// parse!(BooleanValue, executable::boolean_value);
// parse!(NullValue, executable::null_value);
// parse!(EnumValue, executable::enum_value);
// parse!(ListValue, executable::list_value);
// parse!(ObjectValue, executable::object_value);
// parse!(VariableDefinitions, executable::variable_definitions());
// parse!(Arc<VariableDefinition>, executable::variable_definition());
// parse!(Variable, executable::variable);
// parse!(Arc<Type>, executable::ty(RECURSION_LIMIT));
// parse!(NamedType, executable::named_type);
// parse!(NonNullType, executable::non_null_type);
// parse!(Directives, executable::directives);
// parse!(Directive, executable::directive);

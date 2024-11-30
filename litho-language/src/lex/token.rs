use std::borrow::Borrow;
use std::iter::Peekable;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

use logos::Logos;
use unindent::unindent;

use super::raw::{raw_lexer, RawLexer, RawToken};
use super::{SourceId, Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct Error<T>(RawToken<T>);

impl<T> Error<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }

    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        &self.0
    }
}

/// Represents a name in a GraphQL document.
///
/// ```bnf
/// Name ::= NameStart NameContinue*
///
/// NameStart ::= Letter
///
/// NameContinue ::= Letter | Digit
///
/// Letter ::= A | B | C | D | E | F | G | H | I | J | K | L | M |
///            N | O | P | Q | R | S | T | U | V | W | X | Y | Z |
///            a | b | c | d | e | f | g | h | i | j | k | l | m |
///            n | o | p | q | r | s | t | u | v | w | x | y | z
///
/// Digit ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
/// ```
///
/// GraphQL Documents are full of named things: operations, fields, arguments,
/// types directives, fragments, and variables. All names must follow the same
/// grammatical form.
///
/// Names in GraphQL are case-sensitive. That is to say `name`, `Name` and
/// `NAME` all refer to different names.
///
/// A _Name_ must not be followed by a _NameContinue_. In other words, a _Name_
/// token is always the longest possible valid sequence. The source characters
/// `a1` cannot be interpreted as two tokens since `a` is followed by the
/// _NameContinue_ `1`.
///
/// Note: Names in GraphQL are limited to the Latin ASCII subset of
/// _SourceCharacter_ in order to support interoperation with as many other
/// systems as possible.
///
/// ##### Reserved Names
///
/// Any _Name_ within a GraphQL type system must not start with two underscores
/// "__" unless it is part of the introspection system as defined by this
/// specification.
///
/// _Source: [Sec. 2.1.9 Names](https://spec.graphql.org/October2021/#sec-Names)_
#[derive(Clone, Copy, Debug)]
pub struct Name<T>(RawToken<T>);

impl<T> Name<T> {
    pub fn new<'a>(source: &'a str) -> Name<T>
    where
        T: From<&'a str>,
    {
        let kind = match source {
            "query" => TokenKind::KeywordQuery,
            "mutation" => TokenKind::KeywordMutation,
            "subscription" => TokenKind::KeywordSubscription,
            "on" => TokenKind::KeywordOn,
            "fragment" => TokenKind::KeywordFragment,
            "true" => TokenKind::KeywordTrue,
            "false" => TokenKind::KeywordFalse,
            "null" => TokenKind::KeywordNull,
            "schema" => TokenKind::KeywordSchema,
            "extend" => TokenKind::KeywordExtend,
            "scalar" => TokenKind::KeywordScalar,
            "type" => TokenKind::KeywordType,
            "implements" => TokenKind::KeywordImplements,
            "interface" => TokenKind::KeywordInterface,
            "union" => TokenKind::KeywordUnion,
            "enum" => TokenKind::KeywordEnum,
            "input" => TokenKind::KeywordInput,
            "directive" => TokenKind::KeywordDirective,
            "repeatable" => TokenKind::KeywordRepeatable,
            "QUERY" => TokenKind::KeywordDirectiveQuery,
            "MUTATION" => TokenKind::KeywordDirectiveMutation,
            "SUBSCRIPTION" => TokenKind::KeywordDirectiveSubscription,
            "FIELD" => TokenKind::KeywordDirectiveField,
            "FRAGMENT_DEFINITION" => TokenKind::KeywordDirectiveFragmentDefinition,
            "FRAGMENT_SPREAD" => TokenKind::KeywordDirectiveFragmentSpread,
            "INLINE_FRAGMENT" => TokenKind::KeywordDirectiveInlineFragment,
            "VARIABLE_DEFINITION" => TokenKind::KeywordDirectiveVariableDefinition,
            "SCHEMA" => TokenKind::KeywordDirectiveSchema,
            "SCALAR" => TokenKind::KeywordDirectiveScalar,
            "OBJECT" => TokenKind::KeywordDirectiveObject,
            "FIELD_DEFINITION" => TokenKind::KeywordDirectiveFieldDefinition,
            "ARGUMENT_DEFINITION" => TokenKind::KeywordDirectiveArgumentDefinition,
            "INTERFACE" => TokenKind::KeywordDirectiveInterface,
            "UNION" => TokenKind::KeywordDirectiveUnion,
            "ENUM" => TokenKind::KeywordDirectiveEnum,
            "ENUM_VALUE" => TokenKind::KeywordDirectiveEnumValue,
            "INPUT_OBJECT" => TokenKind::KeywordDirectiveInputObject,
            "INPUT_FIELD_DEFINITION" => TokenKind::KeywordDirectiveInputFieldDefinition,
            _ => TokenKind::Name,
        };

        Name(RawToken::new(kind, T::from(source)))
    }

    pub fn span(&self) -> Span {
        self.0.span
    }

    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        &self.0
    }
}

impl<T> AsRef<T> for Name<T> {
    fn as_ref(&self) -> &T {
        self.0.source.borrow()
    }
}

/// Represents a punctuator in a GraphQL document.
///
/// ```bnf
/// Punctuator ::= ! | $ | & | ( | ) | ... | : | = | @ | [ | ] | { | "|" | }
/// ```
///
/// GraphQL documents include punctuation in order to describe structure.
/// GraphQL is a data description language and not a programming language,
/// therefore GraphQL lacks the punctuation often used to describe mathematical
/// expressions.
///
/// _Source: [Sec: 2.1.8 Punctuators](https://spec.graphql.org/October2021/#sec-Punctuators)_
///
/// __Implementation note:__ any punctuator that's not part of the grammar
/// listed above is considered an [Error].
#[derive(Clone, Copy, Debug)]
pub struct Punctuator<T>(RawToken<T>);

impl<T> Punctuator<T> {
    pub fn new(source: &'static str) -> Punctuator<T>
    where
        T: From<&'static str>,
    {
        let kind = match source {
            "&" => TokenKind::Ampersand,
            "@" => TokenKind::At,
            "!" => TokenKind::Bang,
            "{" => TokenKind::BraceLeft,
            "}" => TokenKind::BraceRight,
            "[" => TokenKind::BracketLeft,
            "]" => TokenKind::BracketRight,
            ":" => TokenKind::Colon,
            "$" => TokenKind::Dollar,
            "..." => TokenKind::Dots,
            "=" => TokenKind::Eq,
            "(" => TokenKind::ParenLeft,
            ")" => TokenKind::ParenRight,
            "|" => TokenKind::Pipe,
            _ => todo!(),
        };

        Punctuator(RawToken::new(kind, T::from(source)))
    }

    pub fn span(&self) -> Span {
        self.0.span
    }

    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        &self.0
    }
}

impl<T> AsRef<T> for Punctuator<T> {
    fn as_ref(&self) -> &T {
        self.0.source.borrow()
    }
}

/// Represents an int value (literal) in a GraphQL document.
#[derive(Clone, Copy, Debug)]
pub struct IntValue<T>(RawToken<T>);

impl<T> IntValue<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }

    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        &self.0
    }
}

impl<T> IntValue<T>
where
    T: Borrow<str>,
{
    pub fn to_i32(&self) -> Result<i32, ParseIntError> {
        i32::from_str(self.as_raw_token().source.borrow())
    }
}

/// Represents a float value (literal) in a GraphQL document.
#[derive(Clone, Copy, Debug)]
pub struct FloatValue<T>(RawToken<T>);

impl<T> FloatValue<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }

    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        &self.0
    }
}

impl<T> FloatValue<T>
where
    T: Borrow<str>,
{
    pub fn to_f64(&self) -> Result<f64, ParseFloatError> {
        f64::from_str(self.as_raw_token().source.borrow())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StringValue<T>(RawToken<T>);

impl<T> StringValue<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }

    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        &self.0
    }
}

impl<T> StringValue<T>
where
    for<'a> T: From<&'a str>,
{
    pub fn block<S>(value: S) -> StringValue<T>
    where
        S: Borrow<str>,
    {
        StringValue(RawToken {
            kind: TokenKind::StringValue,
            source: T::from(&format!(
                r#""""{}""""#,
                value.borrow().replace(r#"""""#, "\"\"\"")
            )),
            span: Default::default(),
        })
    }
}

impl<T> ToString for StringValue<T>
where
    T: Borrow<str>,
{
    fn to_string(&self) -> String {
        let source = self.0.source.borrow();

        match source.starts_with("\"\"\"") {
            true => unindent(&source[3..source.len() - 3]),
            false => source[1..source.len() - 1].replace("\\\"", "\""),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Token<T> {
    Error(Error<T>),

    /// Represents a [Name] in GraphQL.
    Name(Name<T>),

    /// Represents a [Punctuator] in GraphQL.
    Punctuator(Punctuator<T>),

    /// Represents an [IntValue] in GraphQL.
    IntValue(IntValue<T>),

    /// Represents a [FloatValue] in GraphQL.
    FloatValue(FloatValue<T>),

    /// Represents a [StringValue] in GraphQL.
    StringValue(StringValue<T>),
}

impl<T> Token<T> {
    #[inline(always)]
    pub fn as_raw_token(&self) -> &RawToken<T> {
        match self {
            Token::Error(error) => error.as_raw_token(),
            Token::Name(name) => name.as_raw_token(),
            Token::Punctuator(punct) => punct.as_raw_token(),
            Token::IntValue(value) => value.as_raw_token(),
            Token::FloatValue(value) => value.as_raw_token(),
            Token::StringValue(value) => value.as_raw_token(),
        }
    }

    #[inline(always)]
    pub fn span(&self) -> Span {
        match self {
            Token::Error(token) => token.0.span,
            Token::Name(token) => token.0.span,
            Token::Punctuator(token) => token.0.span,
            Token::IntValue(token) => token.0.span,
            Token::FloatValue(token) => token.0.span,
            Token::StringValue(token) => token.0.span,
        }
    }
}

impl<T> From<RawToken<T>> for Token<T> {
    #[inline(always)]
    fn from(raw: RawToken<T>) -> Self {
        match raw.kind {
            TokenKind::Error => Token::Error(Error(raw)),
            TokenKind::IntValue => Token::IntValue(IntValue(raw)),
            TokenKind::FloatValue => Token::FloatValue(FloatValue(raw)),
            TokenKind::StringValue => Token::StringValue(StringValue(raw)),
            kind if kind.is_name() => Token::Name(Name(raw)),
            kind if kind.is_punctuator() => Token::Punctuator(Punctuator(raw)),
            _ => unreachable!("All other token types should have been ignored by the lexer."),
        }
    }
}

impl<T> From<Name<T>> for Token<T> {
    fn from(name: Name<T>) -> Self {
        Token::Name(name)
    }
}

pub struct Lexer<'a, T>
where
    T: From<&'a str>,
{
    lexer: Peekable<TokenIter<RawLexer<'a, T>>>,
    last_span: Option<Span>,
}

impl<'a, T> Lexer<'a, T>
where
    T: From<&'a str>,
{
    #[inline(always)]
    pub fn peek(&mut self) -> Option<&Token<T>> {
        self.lexer.peek()
    }

    #[inline(always)]
    pub fn span(&mut self) -> Span {
        match (self.last_span, self.peek()) {
            (Some(left), Some(right)) => Span::between(left, right.span()),
            (Some(left), None) => left.collapse_to_end(),
            (None, Some(right)) => right.span().collapse_to_start(),
            (None, None) => todo!(),
        }
    }
}

impl<'a, T> Iterator for Lexer<'a, T>
where
    T: From<&'a str>,
{
    type Item = Token<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.lexer.next()?;

        self.last_span.replace(token.span());
        Some(token)
    }
}

pub struct TokenIter<I>(I);

impl<I, T> Iterator for TokenIter<I>
where
    I: Iterator<Item = RawToken<T>>,
{
    type Item = Token<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Into::into)
    }
}

pub fn lexer<'a, T>(source_id: SourceId, source: &'a str) -> Lexer<T>
where
    T: From<&'a str>,
{
    let _: <TokenKind as Logos>::Source;

    Lexer {
        lexer: TokenIter(raw_lexer(source_id, TokenKind::lexer(source))).peekable(),
        last_span: Default::default(),
    }
}

mod display {
    use std::borrow::Borrow;
    use std::fmt::{Display, Formatter, Result};

    use super::Name;

    impl<T> Display for Name<T>
    where
        T: Borrow<str>,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0.source.borrow().fmt(f)
        }
    }
}

use std::marker::PhantomData;

use zipped::UnzipFrom;

use nom::error::ParseError;
use nom::{Err, Parser};

use crate::{terminal, Missing};

use super::{opt, skip_unrecognized, Boxed, Input, Opt, Recognizer, Recoverable, SkipUnrecognized};

/// Trait implemented by recoverable parsers, analogous to `nom::Parser`.
pub trait RecoverableParser<I, O, E>: Recognizer<I, E> {
    /// Should return a new `nom::Parser` that can try to parse something up
    /// until the given `recovery_point`.
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
    where
        R: Recognizer<I, E>;

    /// Utility to parse an input entirely and directly to an output.
    fn parse(&self, input: I) -> Result<O, Err<E>>
    where
        I: Input + Clone,
        E: ParseError<I>,
    {
        self.parser(terminal(nom::combinator::eof))
            .parse(input)
            .map(|(_, o)| o)
    }

    /// Returns a recoverable parser that always succeeds, either with the result
    /// of `self` when the underlying parser succeeds, or with
    /// [`Missing::Error`] when it recovers from an underlying parsing error.
    fn recover<M>(self, missing: M) -> Recover<Opt<Self>, M>
    where
        Self: Sized,
        M: Missing<I>,
    {
        Recover(opt(self), missing)
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. The second
    /// parser will be used as a recovery point for the first parser.
    fn and<P>(self, parser: P) -> And<Self, SkipUnrecognized<P>>
    where
        Self: Sized,
    {
        And(self, skip_unrecognized(parser))
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. The second
    /// parser will be used as a recovery point for the first parser.
    fn and_opt<P>(self, parser: P) -> And<Self, Opt<SkipUnrecognized<P>>>
    where
        Self: Sized,
    {
        And(self, opt(skip_unrecognized(parser)))
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. Unlike
    /// [`RecoverableParser::and`], this returns a parser that also recognizes
    /// the `parser` instead of only `self`.
    fn and_recognize<P>(self, parser: P) -> AndRecognize<And<Self, SkipUnrecognized<P>>>
    where
        Self: Sized,
    {
        AndRecognize(self.and(parser))
    }

    /// Returns a recoverable parser that succeeds if `self` succeeds and that
    /// will call the given `missing` function to construct a substitute when
    /// `parser` fails.
    fn and_recover<P, R, M>(
        self,
        parser: P,
        recovery: R,
    ) -> AndRecover<And<Self, Opt<SkipUnrecognized<P>>>, R>
    where
        Self: Sized,
        I: Input,
        R: Fn(&O) -> M,
        M: Missing<I>,
    {
        AndRecover(And(self, opt(skip_unrecognized(parser))), recovery)
    }

    /// Maps the output of this recoverable parser to another type using the
    /// given `apply` closure.
    fn map<F>(self, apply: F) -> Map<Self, O, F>
    where
        Self: Sized,
    {
        Map(self, PhantomData, apply)
    }

    /// Unzips the output of this parser, i.e. maps `((((A, B), C), ...), Z)` to
    /// `(A, B, C, ..., Z)`.
    fn unzip<N>(self) -> impl RecoverableParser<I, N, E>
    where
        Self: Sized,
        I: Input,
        N: UnzipFrom<O>,
    {
        self.map(N::unzip_from)
    }

    /// Returns a boxed (i.e. heap-allocated and type-erased) wrapper around
    /// this parser.
    fn boxed<'a>(self) -> Boxed<'a, I, O, E>
    where
        Self: Sized + 'a,
    {
        Boxed::new(self)
    }
}

impl<I, O, E, P> RecoverableParser<I, O, E> for &P
where
    P: RecoverableParser<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        (*self).parser(recovery_point)
    }
}

pub struct And<A, B>(A, B);

impl<I, E, A, B> Recognizer<I, E> for And<A, B>
where
    A: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, AO, BO, E, A, B> RecoverableParser<I, (AO, BO), E> for And<A, B>
where
    I: Input + Clone,
    E: ParseError<I>,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        move |input| {
            self.0
                .parser((&self.1).or(&recovery_point))
                .and(self.1.parser(&recovery_point))
                .parse(input)
        }
    }
}

pub struct AndRecognize<P>(P);

impl<I, E, A, B> Recognizer<I, E> for AndRecognize<And<A, B>>
where
    I: Clone,
    E: ParseError<I>,
    A: Recognizer<I, E>,
    B: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        let And(a, b) = &self.0;
        a.recognizer().or(b.recognizer())
    }
}

impl<I, AO, BO, E, A, B> RecoverableParser<I, (AO, BO), E> for AndRecognize<And<A, B>>
where
    I: Input + Clone,
    E: ParseError<I>,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        self.0.parser(recovery_point)
    }
}

pub struct AndRecover<P, M>(P, M);

impl<I, E, P, M> Recognizer<I, E> for AndRecover<P, M>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, AO, BO, E, P, F, M> RecoverableParser<I, (AO, Recoverable<BO, M::Error>), E>
    for AndRecover<P, F>
where
    I: Clone + Input,
    E: ParseError<I>,
    P: RecoverableParser<I, (AO, Option<BO>), E>,
    F: Fn(&AO) -> M,
    M: Missing<I>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, (AO, Recoverable<BO, M::Error>), E>
    where
        R: Recognizer<I, E>,
    {
        let mut parser = self.0.parser(recovery_point);

        move |input| {
            let (input, (a, b)) = parser.parse(input)?;

            let b = match b {
                Some(value) => Recoverable::Present(value),
                None => Recoverable::Missing(self.1(&a).error(&input)),
            };

            Ok((input, (a, b)))
        }
    }
}

pub struct Recover<P, M>(P, M);

impl<I, E, P, M> Recognizer<I, E> for Recover<P, M>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O, M, E, P> RecoverableParser<I, Recoverable<O, M::Error>, E> for Recover<P, M>
where
    I: Clone + Input,
    M: Missing<I> + Copy,
    E: ParseError<I>,
    P: RecoverableParser<I, Option<O>, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, Recoverable<O, M::Error>, E>
    where
        R: Recognizer<I, E>,
    {
        let mut parser = self.0.parser(recovery_point);

        move |input| {
            let (input, value) = parser.parse(input)?;

            let value = match value {
                Some(value) => Recoverable::Present(value),
                None => Recoverable::Missing(self.1.error(&input)),
            };

            Ok((input, value))
        }
    }
}

pub struct Map<P, O, F>(P, PhantomData<O>, F);

impl<I, E, P, O, F> Recognizer<I, E> for Map<P, O, F>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O2, E, P, O, F> RecoverableParser<I, O2, E> for Map<P, O, F>
where
    P: RecoverableParser<I, O, E>,
    F: Fn(O) -> O2,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O2, E>
    where
        R: Recognizer<I, E>,
    {
        self.0.parser(recovery_point).map(&self.2)
    }
}

use std::marker::PhantomData;

use zipped::UnzipFrom;

use nom::error::ParseError;
use nom::Parser;

use super::combinator::opt;
use super::{Input, Recognizer, Recoverable};

pub trait RecoverableParser<I, O, E>: Recognizer<I, E> {
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
    where
        R: Recognizer<I, E>;

    fn recover(self, missing: I::Missing) -> Recover<Self, I::Missing>
    where
        Self: Sized,
        I: Input,
    {
        Recover(self, missing)
    }

    fn unzip(self) -> Unzip<Self, O>
    where
        Self: Sized,
    {
        Unzip(self, PhantomData)
    }

    fn and<P>(self, parser: P) -> And<Self, P>
    where
        Self: Sized,
    {
        And(self, parser)
    }

    fn and_recognize<P>(self, parser: P) -> AndRecognize<Self, P>
    where
        Self: Sized,
    {
        AndRecognize(self, parser)
    }

    fn and_recover<P, M>(self, parser: P, missing: M) -> AndRecover<Self, P, M>
    where
        Self: Sized,
        I: Input,
        M: Fn(&O) -> I::Missing,
    {
        AndRecover(self, parser, missing)
    }

    fn map<F>(self, apply: F) -> Map<Self, O, F>
    where
        Self: Sized,
    {
        Map(self, PhantomData, apply)
    }

    fn as_ref(&self) -> &Self {
        self
    }
}

impl<I, O, E, P> RecoverableParser<I, O, E> for &P
where
    I: Iterator,
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
    I: Clone + Input,
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
                .parser(recovery_point.by_ref().or(&self.1))
                .and(self.1.parser(&recovery_point))
                .parse(input)
        }
    }
}

pub struct AndRecognize<A, B>(A, B);

impl<I, E, A, B> Recognizer<I, E> for AndRecognize<A, B>
where
    I: Clone,
    E: ParseError<I>,
    A: Recognizer<I, E>,
    B: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer().or(self.1.recognizer())
    }
}

impl<I, AO, BO, E, A, B> RecoverableParser<I, (AO, BO), E> for AndRecognize<A, B>
where
    I: Clone + Input,
    E: ParseError<I>,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        move |input| And(&self.0, &self.1).parser(&recovery_point).parse(input)
    }
}

pub struct AndRecover<A, B, M>(A, B, M);

impl<I, E, A, B, M> Recognizer<I, E> for AndRecover<A, B, M>
where
    A: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, AO, BO, E, A, B, M> RecoverableParser<I, (AO, Recoverable<BO, I::Missing>), E>
    for AndRecover<A, B, M>
where
    I: Clone + Input,
    E: ParseError<I>,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
    M: Fn(&AO) -> I::Missing,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, (AO, Recoverable<BO, I::Missing>), E>
    where
        R: Recognizer<I, E>,
    {
        move |input| {
            let (input, (a, b)) = self
                .0
                .parser(recovery_point.by_ref().or(&self.1))
                .and(opt(&self.1).parser(&recovery_point))
                .parse(input)?;

            let b = match b {
                Some(value) => Recoverable::Present(value),
                None => Recoverable::Missing(input.missing(self.2(&a))),
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

impl<I, O, E, P> RecoverableParser<I, Recoverable<O, I::Missing>, E> for Recover<P, I::Missing>
where
    I: Clone + Input,
    I::Missing: Copy,
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, Recoverable<O, I::Missing>, E>
    where
        R: Recognizer<I, E>,
    {
        move |input| {
            let (input, value) = opt(&self.0).parser(&recovery_point).parse(input)?;

            let value = match value {
                Some(value) => Recoverable::Present(value),
                None => Recoverable::Missing(input.missing(self.1.clone())),
            };

            Ok((input, value))
        }
    }
}

pub struct Unzip<P, O>(P, PhantomData<O>);

impl<I, E, P, O> Recognizer<I, E> for Unzip<P, O>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, N, E, P, O> RecoverableParser<I, N, E> for Unzip<P, O>
where
    N: UnzipFrom<O>,
    P: RecoverableParser<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, N, E>
    where
        R: Recognizer<I, E>,
    {
        move |input| {
            self.0
                .parser(&recovery_point)
                .map(N::unzip_from)
                .parse(input)
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
    I: Iterator,
    P: RecoverableParser<I, O, E>,
    F: Fn(O) -> O2,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O2, E>
    where
        R: Recognizer<I, E>,
    {
        move |input| self.0.parser(&recovery_point).map(&self.2).parse(input)
    }
}

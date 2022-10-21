use std::marker::PhantomData;

use detuple::FromNested;

use nom::error::{ErrorKind, ParseError};
use nom::multi::many_till;
use nom::{Err, IResult, Parser};

use super::{next, Input, Recognizer, Recoverable};

pub trait RecoverableParser<I, O, E>: Recognizer<I, E> {
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
    where
        R: Recognizer<I, E>;

    fn recover(self, missing: I::Missing) -> Recover<Self, I::Missing>
    where
        Self: Sized,
        I: Input,
    {
        Recover(self, missing)
    }

    fn flatten(self) -> Flatten<Self, O>
    where
        Self: Sized,
    {
        Flatten(self, PhantomData)
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
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        (*self).parse(input, recovery_point)
    }
}

pub struct And<A, B>(A, B);

impl<I, E, A, B> Recognizer<I, E> for And<A, B>
where
    A: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, AO, BO, E, A, B> RecoverableParser<I, (AO, BO), E> for And<A, B>
where
    I: Clone + Input,
    E: ParseError<I>,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        let (input, a) = self.0.parse(input, recovery_point.by_ref().or(&self.1))?;

        let (mut input, (rest, b)) = many_till(
            next,
            (|input| self.1.parse(input, recovery_point.by_ref()))
                .map(Some)
                .or({ |input| recovery_point.recognize(input) }.map(|_| None)),
        )
        .parse(input)?;

        let b = match b {
            Some(value) => value,
            None => return Err(Err::Error(E::from_error_kind(input, ErrorKind::Fail))),
        };

        input.extend(rest);

        Ok((input, (a, b)))
    }
}

pub struct AndRecognize<A, B>(A, B);

impl<I, E, A, B> Recognizer<I, E> for AndRecognize<A, B>
where
    I: Clone,
    A: Recognizer<I, E>,
    B: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        (&self.0).or(&self.1).recognize(input)
    }
}

impl<I, AO, BO, E, A, B> RecoverableParser<I, (AO, BO), E> for AndRecognize<A, B>
where
    I: Clone + Input,
    E: ParseError<I>,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        And(&self.0, &self.1).parse(input, recovery_point)
    }
}

pub struct AndRecover<A, B, M>(A, B, M);

impl<I, E, A, B, M> Recognizer<I, E> for AndRecover<A, B, M>
where
    A: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
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
    fn parse<R>(
        &self,
        input: I,
        recovery_point: R,
    ) -> IResult<I, (AO, Recoverable<BO, I::Missing>), E>
    where
        R: Recognizer<I, E>,
    {
        let (input, a) = self.0.parse(input, recovery_point.by_ref().or(&self.1))?;

        let (mut input, (rest, b)) = many_till(
            next,
            (|input| self.1.parse(input, recovery_point.by_ref()))
                .map(Some)
                .or({ |input| recovery_point.recognize(input) }.map(|_| None)),
        )
        .parse(input)?;

        let b = match b {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(input.missing(self.2(&a))),
        };

        input.extend(rest);

        Ok((input, (a, b)))
    }
}

pub struct Recover<P, M>(P, M);

impl<I, E, P, M> Recognizer<I, E> for Recover<P, M>
where
    P: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E, P> RecoverableParser<I, Recoverable<O, I::Missing>, E> for Recover<P, I::Missing>
where
    I: Clone + Input,
    I::Missing: Copy,
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, Recoverable<O, I::Missing>, E>
    where
        R: Recognizer<I, E>,
    {
        let (mut input, (rest, value)) = many_till(
            next,
            (|input| self.0.parse(input, recovery_point.by_ref()))
                .map(Some)
                .or({ |input| recovery_point.recognize(input) }.map(|_| None)),
        )
        .parse(input)?;

        input.extend(rest);

        let value = match value {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(input.missing(self.1.clone())),
        };

        Ok((input, value))
    }
}

pub struct Flatten<P, O>(P, PhantomData<O>);

impl<I, E, P, O> Recognizer<I, E> for Flatten<P, O>
where
    P: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, N, E, P, O> RecoverableParser<I, N, E> for Flatten<P, O>
where
    N: FromNested<O>,
    P: RecoverableParser<I, O, E>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, N, E>
    where
        R: Recognizer<I, E>,
    {
        self.0
            .parse(input, recovery_point)
            .map(|(input, value)| (input, N::from_nested(value)))
    }
}

pub struct Map<P, O, F>(P, PhantomData<O>, F);

impl<I, E, P, O, F> Recognizer<I, E> for Map<P, O, F>
where
    P: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O2, E, P, O, F> RecoverableParser<I, O2, E> for Map<P, O, F>
where
    I: Iterator,
    P: RecoverableParser<I, O, E>,
    F: Fn(O) -> O2,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O2, E>
    where
        R: Recognizer<I, E>,
    {
        self.0
            .parse(input, recovery_point)
            .map(|(input, output)| (input, (self.2)(output)))
    }
}

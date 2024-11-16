use std::marker::PhantomData;

use nom::error::ParseError;
use nom::{Err, IResult};

use super::{
    opt, skip_unrecognized, Input, Missing, Opt, Recognizer, Recoverable, SkipUnrecognized,
};
/// Trait implemented by recoverable parsers, analogous to `nom::Parser`.
pub trait RecoverableParser<I, O, E, R>
where
    R: Default + Recognizer<I, E>,
{
    fn recognizer(&self) -> R;

    /// Should return a new `nom::Parser` that can try to parse something up
    /// until the given `recovery_point`.
    fn parse(&self, input: I, recovery_point: R) -> IResult<I, O, E>;

    /// Utility to parse an input entirely and directly to an output.
    fn parse_simple(&self, input: I) -> Result<O, Err<E>>
    where
        I: Input + Clone,
        E: ParseError<I>,
    {
        self.parse(input, Default::default()).map(|(_, o)| o)
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
    fn and<P, O2>(self, parser: P) -> And<Self, P>
    where
        I: Input + Clone,
        E: ParseError<I>,
        P: RecoverableParser<I, O2, E, R>,
        Self: Sized,
    {
        And(self, parser)
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. Unlike
    /// [`RecoverableParser::and`], this returns a parser that also recognizes
    /// the `parser` instead of only `self`.
    fn and_recognize<P, O2>(self, parser: P) -> AndRecognize<And<Self, P>>
    where
        I: Input + Clone,
        E: ParseError<I>,
        P: RecoverableParser<I, O2, E, R>,
        Self: Sized,
    {
        AndRecognize(self.and(parser))
    }

    /// Returns a recoverable parser that succeeds if `self` succeeds and that
    /// will call the given `missing` function to construct a substitute when
    /// `parser` fails.
    fn and_recover<P, F, M>(
        self,
        parser: P,
        recovery: F,
    ) -> AndRecover<And<Self, Opt<SkipUnrecognized<P>>>, F>
    where
        Self: Sized,
        I: Input,
        F: Fn(&O) -> M,
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
}

impl<I, O, E, P, R> RecoverableParser<I, O, E, R> for &P
where
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
{
    fn recognizer(&self) -> R {
        (*self).recognizer()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, O, E> {
        (*self).parse(input, recovery_point)
    }
}

pub struct And<A, B>(A, B);

impl<I, AO, BO, E, R, A, B> RecoverableParser<I, (AO, BO), E, R> for And<A, B>
where
    I: Input + Clone,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    A: RecoverableParser<I, AO, E, R>,
    B: RecoverableParser<I, BO, E, R>,
{
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        let (input, a) = self
            .0
            .parse(input, self.1.recognizer().or(recovery_point))?;
        let (input, b) = self.1.parse(input, recovery_point)?;

        Ok((input, (a, b)))
    }
}

pub struct AndRecognize<P>(P);

impl<I, AO, BO, E, R, A, B> RecoverableParser<I, (AO, BO), E, R> for AndRecognize<And<A, B>>
where
    I: Input + Clone,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    A: RecoverableParser<I, AO, E, R>,
    B: RecoverableParser<I, BO, E, R>,
{
    fn recognizer(&self) -> R {
        let And(a, b) = &self.0;
        a.recognizer().or(b.recognizer())
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, (AO, BO), E>
    where
        R: Recognizer<I, E>,
    {
        self.0.parse(input, recovery_point)
    }
}

pub struct AndRecover<P, M>(P, M);

impl<I, AO, BO, E, R, P, F, M> RecoverableParser<I, (AO, Recoverable<BO, M::Error>), E, R>
    for AndRecover<P, F>
where
    I: Clone + Input,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, (AO, Option<BO>), E, R>,
    F: Fn(&AO) -> M,
    M: Missing<I>,
{
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, (AO, Recoverable<BO, M::Error>), E> {
        let (input, (a, b)) = self.0.parse(input, recovery_point)?;

        let b = match b {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(self.1(&a).error(&input)),
        };

        Ok((input, (a, b)))
    }
}

pub struct Recover<P, M>(P, M);

impl<I, O, M, E, R, P> RecoverableParser<I, Recoverable<O, M::Error>, E, R> for Recover<P, M>
where
    I: Clone + Input,
    M: Missing<I> + Copy,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, Option<O>, E, R>,
{
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, Recoverable<O, M::Error>, E> {
        let (input, value) = self.0.parse(input, recovery_point)?;

        let value = match value {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(self.1.error(&input)),
        };

        Ok((input, value))
    }
}

pub struct Map<P, O, F>(P, PhantomData<O>, F);

impl<I, O2, E, R, P, O, F> RecoverableParser<I, O2, E, R> for Map<P, O, F>
where
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
    F: Fn(O) -> O2,
{
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, O2, E> {
        self.0
            .parse(input, recovery_point)
            .map(|(input, value)| (input, (self.2)(value)))
    }
}

macro_rules! tuple {
    ($($ident:ident $output:ident)+) => {
        impl<_I, _E, _R, $($ident,)* $($output,)*> RecoverableParser<_I, ($($output,)*), _E, _R>
            for ($($ident,)*)
        where
            _I: Input + Clone,
            _E: ParseError<_I>,
            _R: Recognizer<_I, _E>,
            $($ident: RecoverableParser<_I, $output, _E, _R>,)*
        {
            fn recognizer(&self) -> _R {
                self.0.recognizer()
            }

            fn parse(&self, input: _I, recovery_point: _R) -> IResult<_I, ($($output,)*), _E>
            {
                #[allow(non_snake_case)]
                let (
                    $($ident,)*
                ) = &self;

                tuple!(& recovery_point input $($ident)*);

                Ok((input, ($($ident,)*)))
            }
        }
    };
    (& $recovery_point:ident $input:ident $first:ident $($rest:ident)*) => {
        #[allow(non_snake_case)]
        let ($input, $first) = $first
            .parse($input, $recovery_point
                    $(.or($rest.recognizer()))*)?;

        tuple!(@ $recovery_point $input $($rest)*);
    };
    (@ $recovery_point:ident $input:ident $first:ident $($rest:ident)*) => {
        #[allow(non_snake_case)]
        let ($input, $first) = skip_unrecognized($first)
            .parse($input, $recovery_point
                $(.or($rest.recognizer()))*)?;

        tuple!(@ $recovery_point $input $($rest)*);
    };
    (@ $recovery_point:ident $input:ident) => {};
}

tuple!(A AO B BO C CO);
tuple!(A AO B BO C CO D DO);
tuple!(A AO B BO C CO D DO E EO);
tuple!(A AO B BO C CO D DO E EO F FO);

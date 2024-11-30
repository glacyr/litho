use std::marker::PhantomData;

use super::{opt, Input, Missing, Opt, Recoverable};

/// Trait implemented by recoverable parsers.
pub trait RecoverableParser<I, O, E>
where
    I: Input,
{
    /// Should return a recognizer that can predict the start of this parser's
    /// grammar.
    fn recognizer(&self) -> I::Recognizer;

    /// Should try to parse something up until the given `recovery_point`. Note
    /// that if the parser's [`RecoverableParser::recognizer`] succeeded, this
    /// function should not fail.
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<O, E>;

    /// Returns a recoverable parser that always succeeds, either with the result
    /// of `self` when the underlying parser succeeds, or with
    /// [`Missing::Error`] when it recovers from an underlying parsing error.
    fn recover<M>(self, missing: M) -> Recover<Opt<Self>, M>
    where
        Self: Sized,
        I: Input,
        M: Missing<I>,
    {
        Recover(opt(self), missing)
    }

    /// Returns a recoverable parser that passes its output to the given
    /// function and then invokes the parser returned by that function with the
    /// remaining input.
    fn flat_map<F, P, O2>(self, parser_fn: F) -> FlatMap<Self, O, F>
    where
        Self: Sized,
        F: FnMut(O) -> P,
        P: RecoverableParser<I, O2, E>,
    {
        FlatMap(self, PhantomData, parser_fn)
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. The second
    /// parser will be used as a recovery point for the first parser.
    fn and<P, O2>(self, parser: P) -> (Self, P)
    where
        I: Input,
        P: RecoverableParser<I, O2, E>,
        Self: Sized,
    {
        (self, parser)
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. Unlike
    /// [`RecoverableParser::and`], this returns a parser that also recognizes
    /// the `parser` instead of only `self`.
    fn and_recognize<P, O2>(self, parser: P) -> AndRecognize<(Self, P)>
    where
        I: Input,
        P: RecoverableParser<I, O2, E>,
        Self: Sized,
    {
        AndRecognize(self.and(parser))
    }

    /// Returns a recoverable parser that succeeds if `self` succeeds and that
    /// will call the given `missing` function to construct a substitute when
    /// `parser` fails.
    fn and_recover<P, O2, F, M>(self, parser: P, recovery: F) -> AndRecover<(Self, Opt<P>), F>
    where
        Self: Sized,
        P: RecoverableParser<I, O2, E>,
        I: Input,
        F: Fn(&O) -> M,
        M: Missing<I>,
    {
        AndRecover((self, opt(parser)), recovery)
    }

    /// Maps the output of this recoverable parser to another type using the
    /// given `apply` closure.
    fn map<F, O2>(self, apply: F) -> Map<Self, O, F>
    where
        Self: Sized,
        F: FnMut(O) -> O2,
    {
        Map(self, PhantomData, apply)
    }

    /// Turns an optional output into a fatal error with the given function.
    fn ok_or_else<F>(self, err: F) -> OkOrElse<Self, F>
    where
        Self: Sized,
    {
        OkOrElse(self, err)
    }
}

pub struct FlatMap<P, O, F>(P, PhantomData<O>, F);

impl<I, O, O2, E, P, P2, F> RecoverableParser<I, O2, E> for FlatMap<P, O, F>
where
    I: Input,
    P: RecoverableParser<I, O, E>,
    P2: RecoverableParser<I, O2, E>,
    F: FnMut(O) -> P2,
    O: Clone,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<O2, E> {
        let output = self.0.parse(input, recovery_point)?;
        let mut parser = (self.2)(output.clone());
        let result = parser.parse(input, recovery_point)?;

        Ok(result)
    }
}

pub struct AndRecognize<P>(P);

impl<I, AO, BO, E, A, B> RecoverableParser<I, (AO, BO), E> for AndRecognize<(A, B)>
where
    I: Input,
    A: RecoverableParser<I, AO, E>,
    B: RecoverableParser<I, BO, E>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        let (a, b) = &self.0;
        a.recognizer() | b.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<(AO, BO), E> {
        self.0.parse(input, recovery_point)
    }
}

pub struct AndRecover<P, M>(P, M);

impl<I, AO, BO, E, P, F, M> RecoverableParser<I, (AO, Recoverable<BO, M::Error>), E>
    for AndRecover<P, F>
where
    I: Input,
    P: RecoverableParser<I, (AO, Option<BO>), E>,
    F: Fn(&AO) -> M,
    M: Missing<I>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(
        &mut self,
        input: &mut I,
        recovery_point: I::Recognizer,
    ) -> Result<(AO, Recoverable<BO, M::Error>), E> {
        let (a, b) = self.0.parse(input, recovery_point)?;

        let b = match b {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(self.1(&a).error(input)),
        };

        Ok((a, b))
    }
}

pub struct Recover<P, M>(P, M);

impl<I, O, M, E, P> RecoverableParser<I, Recoverable<O, M::Error>, E> for Recover<P, M>
where
    I: Input,
    M: Missing<I> + Copy,
    P: RecoverableParser<I, Option<O>, E>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(
        &mut self,
        input: &mut I,
        recovery_point: I::Recognizer,
    ) -> Result<Recoverable<O, M::Error>, E> {
        let value = self.0.parse(input, recovery_point)?;

        let value = match value {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(self.1.error(input)),
        };

        Ok(value)
    }
}

pub struct Map<P, O, F>(P, PhantomData<O>, F);

impl<I, O2, E, P, O, F> RecoverableParser<I, O2, E> for Map<P, O, F>
where
    I: Input,
    P: RecoverableParser<I, O, E>,
    F: FnMut(O) -> O2,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<O2, E> {
        self.0.parse(input, recovery_point).map(&mut self.2)
    }
}

pub struct OkOrElse<P, F>(P, F);

impl<I, O, E, P, F> RecoverableParser<I, O, E> for OkOrElse<P, F>
where
    I: Input,
    P: RecoverableParser<I, Option<O>, E>,
    F: FnMut(&mut I) -> E,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<O, E> {
        self.0
            .parse(input, recovery_point)?
            .ok_or_else(|| self.1(input))
    }
}

macro_rules! tuple {
    ($($ident:ident $output:ident)+) => {
        impl<_I, _E, $($ident,)* $($output,)*> RecoverableParser<_I, ($($output,)*), _E>
            for ($($ident,)*)
        where
            _I: Input,
            $($ident: RecoverableParser<_I, $output, _E>,)*
        {
            #[inline(always)]
            fn recognizer(&self) -> _I::Recognizer {
                self.0.recognizer()
            }

            #[inline(always)]
            fn parse(&mut self, input: &mut _I, recovery_point: _I::Recognizer) -> Result<($($output,)*), _E>
            {
                #[allow(non_snake_case)]
                let (
                    $($ident,)*
                ) = self;

                tuple!(& recovery_point input $($ident)*);

                Ok(($($ident,)*))
            }
        }
    };
    (& $recovery_point:ident $input:ident $first:ident $($rest:ident)*) => {
        #[allow(non_snake_case)]
        let $first = $first
            .parse($input, $recovery_point
                    $(| $rest.recognizer())*)?;

        tuple!(@ $recovery_point $input $($rest)*);
    };
    (@ $recovery_point:ident $input:ident $first:ident $($rest:ident)*) => {
        #[allow(non_snake_case)]
        let $first = $first
            .parse($input, $recovery_point
                $(| $rest.recognizer())*)?;

        tuple!(@ $recovery_point $input $($rest)*);
    };
    (@ $recovery_point:ident $input:ident) => {};
}

tuple!(A AO B BO);
tuple!(A AO B BO C CO);
tuple!(A AO B BO C CO D DO);
tuple!(A AO B BO C CO D DO E EO);
tuple!(A AO B BO C CO D DO E EO F FO);

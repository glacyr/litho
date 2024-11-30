use std::marker::PhantomData;

use nom::error::ParseError;

use super::{opt, Input, Missing, Opt, Recognizer, Recoverable};

/// Trait implemented by recoverable parsers.
pub trait RecoverableParser<I, O, E, R>
where
    R: Default + Recognizer<I, E>,
{
    /// Should return a recognizer that can predict the start of this parser's
    /// grammar.
    fn recognizer(&self) -> R;

    /// Should try to parse something up until the given `recovery_point`. Note
    /// that if the parser's [`RecoverableParser::recognizer`] succeeded, this
    /// function should not fail.
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<O, E>;

    /// Utility to parse an input entirely and directly to an output.
    fn parse_simple(&mut self, input: &mut I) -> Result<O, E>
    where
        I: Input,
        E: ParseError<I>,
    {
        self.parse(input, Default::default())
    }

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
        P: RecoverableParser<I, O2, E, R>,
    {
        FlatMap(self, PhantomData, parser_fn)
    }

    /// Returns a recoverable parser that succeeds if both `self` and the given
    /// `parser` succeed, and returns the output of both in a tuple. The second
    /// parser will be used as a recovery point for the first parser.
    fn and<P, O2>(self, parser: P) -> (Self, P)
    where
        I: Input,
        P: RecoverableParser<I, O2, E, R>,
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
        E: ParseError<I>,
        P: RecoverableParser<I, O2, E, R>,
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
        P: RecoverableParser<I, O2, E, R>,
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
}

pub struct FlatMap<P, O, F>(P, PhantomData<O>, F);

impl<I, O, O2, E, R, P, P2, F> RecoverableParser<I, O2, E, R> for FlatMap<P, O, F>
where
    I: Input,
    E: for<'a> ParseError<&'a I>,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
    P2: RecoverableParser<I, O2, E, R>,
    F: FnMut(O) -> P2,
    O: Clone,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<O2, E> {
        let output = self.0.parse(input, recovery_point)?;
        let mut parser = (self.2)(output.clone());
        let result = parser.parse(input, recovery_point)?;

        Ok(result)
    }
}

pub struct AndRecognize<P>(P);

impl<I, AO, BO, E, R, A, B> RecoverableParser<I, (AO, BO), E, R> for AndRecognize<(A, B)>
where
    I: Input,
    R: Recognizer<I, E>,
    A: RecoverableParser<I, AO, E, R>,
    B: RecoverableParser<I, BO, E, R>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        let (a, b) = &self.0;
        a.recognizer().or(b.recognizer())
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<(AO, BO), E>
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
    I: Input,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, (AO, Option<BO>), E, R>,
    F: Fn(&AO) -> M,
    M: Missing<I>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(
        &mut self,
        input: &mut I,
        recovery_point: R,
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

impl<I, O, M, E, R, P> RecoverableParser<I, Recoverable<O, M::Error>, E, R> for Recover<P, M>
where
    I: Input,
    M: Missing<I> + Copy,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, Option<O>, E, R>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<Recoverable<O, M::Error>, E> {
        let value = self.0.parse(input, recovery_point)?;

        let value = match value {
            Some(value) => Recoverable::Present(value),
            None => Recoverable::Missing(self.1.error(input)),
        };

        Ok(value)
    }
}

pub struct Map<P, O, F>(P, PhantomData<O>, F);

impl<I, O2, E, R, P, O, F> RecoverableParser<I, O2, E, R> for Map<P, O, F>
where
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
    F: FnMut(O) -> O2,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<O2, E> {
        self.0.parse(input, recovery_point).map(&mut self.2)
    }
}

macro_rules! tuple {
    ($($ident:ident $output:ident)+) => {
        impl<_I, _E, _R, $($ident,)* $($output,)*> RecoverableParser<_I, ($($output,)*), _E, _R>
            for ($($ident,)*)
        where
            _I: Input,
            _R: Recognizer<_I, _E>,
            $($ident: RecoverableParser<_I, $output, _E, _R>,)*
        {
            #[inline(always)]
            fn recognizer(&self) -> _R {
                self.0.recognizer()
            }

            #[inline(always)]
            fn parse(&mut self, input: &mut _I, recovery_point: _R) -> Result<($($output,)*), _E>
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
                    $(.or($rest.recognizer()))*)?;

        tuple!(@ $recovery_point $input $($rest)*);
    };
    (@ $recovery_point:ident $input:ident $first:ident $($rest:ident)*) => {
        #[allow(non_snake_case)]
        let $first = $first
            .parse($input, $recovery_point
                $(.or($rest.recognizer()))*)?;

        tuple!(@ $recovery_point $input $($rest)*);
    };
    (@ $recovery_point:ident $input:ident) => {};
}

tuple!(A AO B BO);
tuple!(A AO B BO C CO);
tuple!(A AO B BO C CO D DO);
tuple!(A AO B BO C CO D DO E EO);
tuple!(A AO B BO C CO D DO E EO F FO);

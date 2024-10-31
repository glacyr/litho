use nom::{IResult, Parser};

use super::{Recognizer, RecoverableParser};

/// Boxed (i.e. heap-allocated and type-erased) wrapper around a parser.
pub struct Boxed<'a, I, O, E>(Box<dyn erased::ErasedRecoverableParser<I, O, E> + 'a>);

impl<'a, I, O, E> Boxed<'a, I, O, E> {
    /// Returns a new boxed (i.e. heap-allocated and type-erased) wrapper around
    /// the given `parser`.
    pub fn new<P>(parser: P) -> Boxed<'a, I, O, E>
    where
        P: RecoverableParser<I, O, E> + 'a,
    {
        Boxed(Box::new(parser))
    }
}

impl<I, O, E> Recognizer<I, E> for Boxed<'_, I, O, E> {
    fn recognizer(&self) -> impl Parser<I, (), E> {
        |input| self.0.recognize(input)
    }
}

impl<I, O, E> RecoverableParser<I, O, E> for Boxed<'_, I, O, E> {
    fn parser<R2>(&self, recovery_point: R2) -> impl Parser<I, O, E>
    where
        R2: Recognizer<I, E>,
    {
        move |input| self.0.parse(input, &recovery_point)
    }
}

mod erased {
    use super::{IResult, Parser, Recognizer, RecoverableParser};

    pub trait ErasedRecognizer<I, E> {
        fn recognize(&self, input: I) -> IResult<I, (), E>;
    }

    impl<'a, I, E> Recognizer<I, E> for dyn ErasedRecognizer<I, E> + 'a {
        fn recognizer(&self) -> impl Parser<I, (), E> {
            |input| self.recognize(input)
        }
    }

    impl<I, E, R> ErasedRecognizer<I, E> for R
    where
        R: Recognizer<I, E>,
    {
        fn recognize(&self, input: I) -> IResult<I, (), E> {
            Recognizer::recognizer(&self).parse(input)
        }
    }

    pub trait ErasedRecoverableParser<I, O, E>: ErasedRecognizer<I, E> {
        fn parse(&self, input: I, recovery_point: &dyn ErasedRecognizer<I, E>) -> IResult<I, O, E>;
    }

    impl<I, O, E, P> ErasedRecoverableParser<I, O, E> for P
    where
        P: RecoverableParser<I, O, E>,
    {
        fn parse(&self, input: I, recovery_point: &dyn ErasedRecognizer<I, E>) -> IResult<I, O, E> {
            let mut parser = self.parser(recovery_point);
            parser.parse(input)
        }
    }
}

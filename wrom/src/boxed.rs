use nom::IResult;

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
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E> RecoverableParser<I, O, E> for Boxed<'_, I, O, E> {
    fn parse<R2>(&self, input: I, recovery_point: R2) -> IResult<I, O, E>
    where
        R2: Recognizer<I, E>,
    {
        self.0.parse(input, &recovery_point)
    }
}

mod erased {
    use super::{IResult, Recognizer, RecoverableParser};

    pub trait ErasedRecoverableParser<I, O, E>: Recognizer<I, E> {
        fn parse(&self, input: I, recovery_point: &dyn Recognizer<I, E>) -> IResult<I, O, E>;
    }

    impl<I, O, E, P> ErasedRecoverableParser<I, O, E> for P
    where
        P: RecoverableParser<I, O, E>,
    {
        fn parse(&self, input: I, recovery_point: &dyn Recognizer<I, E>) -> IResult<I, O, E> {
            RecoverableParser::parse(self, input, recovery_point)
        }
    }
}

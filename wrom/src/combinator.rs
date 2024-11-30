use super::{Input, RecoverableParser};

/// Recoverable parser that always succeeds, either with `Some(_)` if the given
/// underlying parser succeeds or with `None` if the underlying parser fails.
pub struct Opt<P>(P);

impl<I, O, E, P> RecoverableParser<I, Option<O>, E> for Opt<P>
where
    I: Input,
    P: RecoverableParser<I, O, E>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<Option<O>, E> {
        input
            .discard(self.0.recognizer(), recovery_point)
            .then(|| self.0.parse(input, recovery_point))
            .transpose()
    }
}

/// Returns a recoverable parser that always succeeds, with `Some(_)` if the
/// given `parser` succeeds and with `None` if the given `parser` fails.
pub fn opt<P>(parser: P) -> Opt<P> {
    Opt(parser)
}

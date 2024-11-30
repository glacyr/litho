use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that always succeeds, either with `Some(_)` if the given
/// underlying parser succeeds or with `None` if the underlying parser fails.
pub struct Opt<P>(P);

impl<I, O, E, R, P> RecoverableParser<I, Option<O>, E, R> for Opt<P>
where
    I: Input,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<Option<O>, E> {
        loop {
            // Invoke the parser when its recognizer succeeds.
            if self.0.recognizer().recognize(input).is_ok() {
                break self.0.parse(input, recovery_point).map(Some);
            }

            // Back out when the recovery point is recognized.
            if recovery_point.recognize(input).is_ok() {
                break Ok(None);
            }

            // Feed back any unrecognized tokens.
            match input.next() {
                Some(token) => input.unrecognized(token),
                None => break Ok(None),
            }
        }
    }
}

/// Returns a recoverable parser that always succeeds, with `Some(_)` if the
/// given `parser` succeeds and with `None` if the given `parser` fails.
pub fn opt<P>(parser: P) -> Opt<P> {
    Opt(parser)
}

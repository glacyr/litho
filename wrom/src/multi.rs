use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that invokes an underlying parser multiple times and
/// returns a `Vec<T>` of results.
pub struct Many<P>(P);

impl<I, O, E, R, P> RecoverableParser<I, Vec<O>, E, R> for Many<P>
where
    I: Input,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline(never)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<Vec<O>, E>
    where
        R: Recognizer<I, E>,
    {
        let mut results = Vec::new();

        let next_recovery_point = recovery_point.or(self.0.recognizer());

        loop {
            if self.0.recognizer().recognize(input).is_ok() {
                let result = self.0.parse(input, next_recovery_point)?;
                results.push(result);
                continue;
            }

            if recovery_point.recognize(input).is_ok() {
                break;
            }

            if let Some(token) = input.next() {
                input.unrecognized(token);
            } else {
                break;
            }
        }

        Ok(results)
    }
}

/// Returns a recoverable parser that invokes the given `parser` multiple times
/// and returns a `Vec<T>` of results, while skipping any unrecognized tokens
/// that precede each result.
pub fn many<P>(parser: P) -> Many<P> {
    Many(parser)
}

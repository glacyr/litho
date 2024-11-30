use super::{Input, RecoverableParser};

/// Recoverable parser that invokes an underlying parser multiple times and
/// returns a `Vec<T>` of results.
pub struct Many<P>(P);

impl<I, O, E, P> RecoverableParser<I, Vec<O>, E> for Many<P>
where
    I: Input,
    P: RecoverableParser<I, O, E>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(never)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<Vec<O>, E> {
        let mut results = Vec::new();

        let next_recovery_point = recovery_point | self.0.recognizer();

        while input.discard(self.0.recognizer(), recovery_point) {
            let result = self.0.parse(input, next_recovery_point)?;
            results.push(result);
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

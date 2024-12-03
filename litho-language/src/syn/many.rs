use std::iter::once;

use crate::ast::List;

use super::{Context, ContextValue, Input, RecoverableParser};

/// Recoverable parser that invokes an underlying parser multiple times and
/// returns a `Vec<T>` of results.
pub struct ManyExt<P>(P);

impl<'a, I, T, O, E, P> RecoverableParser<I, List<'a, T, O>, E> for ManyExt<P>
where
    I: Input + Context<'a, T>,
    O: Clone + 'a,
    T: ContextValue<'a>,
    P: RecoverableParser<I, O, E>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<List<'a, T, O>, E> {
        let mut results = input.list();

        let next_recovery_point = recovery_point | self.0.recognizer();

        while input.discard(self.0.recognizer(), recovery_point) {
            let result = self.0.parse(input, next_recovery_point)?;
            results.extend(once(result));
        }

        Ok(results)
    }
}

/// Returns a recoverable parser that invokes the given `parser` multiple times
/// and returns a `Vec<T>` of results, while skipping any unrecognized tokens
/// that precede each result.
pub fn many_ext<P>(parser: P) -> ManyExt<P> {
    ManyExt(parser)
}

use super::{Input, Recognizer, RecoverableParser};

pub struct Recursive<F>(usize, F);

/// Returns a recursive wrapper that breaks infinite recursion and prevents
/// stack overflows.
///
/// The returned recursive parser serves two purposes: first it lazily calls the
/// given `parser_fn` instead of calling it immediately (breaking infinite
/// recursion), and it keeps track of the recursion depth and returns an error
/// when `max_depth` is reached, thereby preventing a stack overflow.
pub fn recursive<F>(max_depth: usize, parser_fn: F) -> Recursive<F> {
    Recursive(max_depth, parser_fn)
}

impl<I, O, E, R, F, P> RecoverableParser<I, Option<O>, E, R> for Recursive<F>
where
    F: Fn(usize) -> P,
    P: RecoverableParser<I, O, E, R>,
    I: Input,
    R: Recognizer<I, E>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        (self.1)(self.0).recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<Option<O>, E> {
        match self.0 {
            0 => Ok(None),
            n => (self.1)(n - 1).parse(input, recovery_point).map(Some),
        }
    }
}

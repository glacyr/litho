use super::{opt, Input, Missing, Opt, Recognizer, Recoverable, RecoverableParser};

pub struct Delimited<F, G, H, J>(F, G, H, J);

impl<F, G, H, J, I, FO, GO, HO, M, E, R>
    RecoverableParser<I, (FO, GO, Recoverable<HO, M::Error>), E, R> for Delimited<F, G, H, J>
where
    I: Input,
    R: Recognizer<I, E>,
    F: RecoverableParser<I, FO, E, R>,
    G: RecoverableParser<I, GO, E, R>,
    H: RecoverableParser<I, Option<HO>, E, R>,
    J: Fn(&FO) -> M,
    M: Missing<I>,
{
    #[inline(always)]
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    #[inline]
    fn parse(
        &mut self,
        input: &mut I,
        recovery_point: R,
    ) -> Result<(FO, GO, Recoverable<HO, M::Error>), E> {
        let a = self.0.parse(
            input,
            recovery_point
                .or(self.1.recognizer())
                .or(self.2.recognizer()),
        )?;
        let b = self
            .1
            .parse(input, recovery_point.or(self.2.recognizer()))?;
        let c = self.2.parse(input, recovery_point)?;

        let c = match c {
            Some(c) => Recoverable::Present(c),
            None => Recoverable::Missing(self.3(&a).error(input)),
        };

        Ok((a, b, c))
    }
}

/// Returns a recoverable parser that runs the `first`, `second` and `third`
/// parser in succession, and uses the `recovery` rule if the `third` parser
/// fails.
pub fn delimited<F, G, H, J>(
    first: F,
    second: G,
    third: H,
    recovery: J,
) -> Delimited<F, G, Opt<H>, J> {
    Delimited(first, second, opt(third), recovery)
}

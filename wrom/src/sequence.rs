use nom::error::ParseError;

use super::{Input, Recoverable, RecoverableParser};

pub fn delimited<F, G, H, R, I, FO, GO, HO, E>(
    first: F,
    second: G,
    third: H,
    recovery: R,
) -> impl RecoverableParser<I, (FO, GO, Recoverable<HO, I::Missing>), E>
where
    I: Input,
    F: RecoverableParser<I, FO, E>,
    G: RecoverableParser<I, GO, E>,
    H: RecoverableParser<I, HO, E>,
    R: Fn(&FO) -> I::Missing,
    E: ParseError<I>,
{
    first
        .and(second)
        .and_recover(third, move |(left, _)| recovery(left))
        .flatten()
}

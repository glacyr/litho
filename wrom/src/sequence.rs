use nom::error::ParseError;

use super::{Input, Missing, Recoverable, RecoverableParser};

/// Returns a recoverable parser that runs the `first`, `second` and `third`
/// parser in succession, and uses the `recovery` rule if the `third` parser
/// fails.
pub fn delimited<F, G, H, R, I, FO, GO, HO, M, E>(
    first: F,
    second: G,
    third: H,
    recovery: R,
) -> impl RecoverableParser<I, (FO, GO, Recoverable<HO, M::Error>), E>
where
    I: Input + Clone,
    F: RecoverableParser<I, FO, E>,
    G: RecoverableParser<I, GO, E>,
    H: RecoverableParser<I, HO, E>,
    R: Fn(&FO) -> M,
    M: Missing<I>,
    E: ParseError<I>,
{
    first
        .and(second)
        .and_recover(third, move |(left, _)| recovery(left))
        .unzip()
}

#[cfg(test)]
mod tests {
    use nom::combinator::eof;
    use nom::Parser;

    use crate::mock::{char, CollectUnrecognized};
    use crate::{terminal, Recoverable, RecoverableParser};

    use super::delimited;

    #[test]
    pub fn test_delimited() {
        let input = CollectUnrecognized::new("1-,-32");

        let one = char::<_, ()>('1');
        let comma = char(',');
        let two = char('2');
        let three = char('3');

        let grammar = delimited(one, comma, two, |_| ());

        let (mut input, (a, b, c)) = grammar.parser(&terminal(eof)).parse(input).unwrap();
        assert_eq!(a, '1');
        assert_eq!(b, ',');
        assert_eq!(c, Recoverable::Present('2'));
        assert_eq!(
            input.unrecognized().collect::<Vec<_>>(),
            vec!['-', '-', '3']
        );
        assert_eq!(input.into_inner(), "");

        let input = CollectUnrecognized::new("1-,-32");

        let (mut input, (a, b, c)) = grammar.parser(&three).parse(input).unwrap();
        assert_eq!(a, '1');
        assert_eq!(b, ',');
        assert_eq!(c, Recoverable::Missing(()));
        assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['-']);
        assert_eq!(input.into_inner(), "-32");
    }
}

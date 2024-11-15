use nom::error::ParseError;

use super::{Input, Missing, Recognizer, Recoverable, RecoverableParser};

/// Returns a recoverable parser that runs the `first`, `second` and `third`
/// parser in succession, and uses the `recovery` rule if the `third` parser
/// fails.
pub fn delimited<F, G, H, J, I, FO, GO, HO, M, E, R>(
    first: F,
    second: G,
    third: H,
    recovery: J,
) -> impl RecoverableParser<I, (FO, GO, Recoverable<HO, M::Error>), E, R>
where
    I: Input + Clone,
    R: Recognizer<I, E>,
    F: RecoverableParser<I, FO, E, R>,
    G: RecoverableParser<I, GO, E, R>,
    H: RecoverableParser<I, HO, E, R>,
    J: Fn(&FO) -> M,
    M: Missing<I>,
    E: ParseError<I>,
{
    first
        .and(second)
        .and_recover(third, move |(left, _)| recovery(left))
        .map(|((a, b), c)| (a, b, c))
}

// #[cfg(test)]
// mod tests {
//     use nom::combinator::eof;

//     use crate::mock::{char, CollectUnrecognized};
//     use crate::{terminal, Recoverable, RecoverableParser};

//     use super::delimited;

//     #[test]
//     pub fn test_delimited() {
//         let input = CollectUnrecognized::new("1-,-32");

//         let one = char::<_, ()>('1');
//         let comma = char(',');
//         let two = char('2');
//         let three = char('3');

//         let grammar = delimited(one, comma, two, |_| ());

//         let (mut input, (a, b, c)) = grammar.parse(input, &terminal(eof)).unwrap();
//         assert_eq!(a, '1');
//         assert_eq!(b, ',');
//         assert_eq!(c, Recoverable::Present('2'));
//         assert_eq!(
//             input.unrecognized().collect::<Vec<_>>(),
//             vec!['-', '-', '3']
//         );
//         assert_eq!(input.into_inner(), "");

//         let input = CollectUnrecognized::new("1-,-32");

//         let (mut input, (a, b, c)) = grammar.parse(input, &three).unwrap();
//         assert_eq!(a, '1');
//         assert_eq!(b, ',');
//         assert_eq!(c, Recoverable::Missing(()));
//         assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['-']);
//         assert_eq!(input.into_inner(), "-32");
//     }
// }

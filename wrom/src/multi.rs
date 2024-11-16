use nom::error::ParseError;
use nom::{IResult, Parser};

use super::skip::{skip_unrecognized, SkipUnrecognized};
use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that invokes an underlying parser multiple times and
/// returns a `Vec<T>` of results.
pub struct Many<P>(P, bool);

impl<I, O, E, R, P> RecoverableParser<I, Vec<O>, E, R> for Many<P>
where
    I: Clone + Input,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
{
    fn recognizer(&self) -> R {
        self.0.recognizer()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, Vec<O>, E>
    where
        R: Recognizer<I, E>,
    {
        let parser = move |input| self.0.parse(input, recovery_point.or(self.0.recognizer()));

        match self.1 {
            false => nom::multi::many0(parser).parse(input),
            true => nom::multi::many1(parser).parse(input),
        }
    }
}

/// Returns a recoverable parser that invokes the given `parser` multiple times
/// and returns a `Vec<T>` of results, while skipping any unrecognized tokens
/// that precede each result. This parser does not fail when the result set is
/// empty.
pub fn many0<P>(parser: P) -> Many<SkipUnrecognized<P>> {
    Many(skip_unrecognized(parser), false)
}

/// Returns a recoverable parser that invokes the given `parser` one or more
/// times and returns a `Vec<T>` of results, while skipping any unrecognized
/// tokens that precede each result. This parser fails when the result set is
/// empty.
pub fn many1<P>(parser: P) -> Many<SkipUnrecognized<P>> {
    Many(skip_unrecognized(parser), true)
}

// #[cfg(test)]
// mod tests {
//     use nom::combinator::eof;

//     use crate::mock::{char, CollectUnrecognized};
//     use crate::{terminal, RecoverableParser};

//     use super::{many0, many1};

//     #[test]
//     pub fn test_many0() {
//         let input = CollectUnrecognized::new("aba");
//         let one = many0(char::<_, ()>('a'));
//         let eof = terminal(eof);

//         let (mut input, tokens) = one.parse(input, &eof).unwrap();
//         assert_eq!(tokens, vec!['a', 'a']);

//         assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['b']);
//     }

//     #[test]
//     pub fn test_many0_empty() {
//         let input = CollectUnrecognized::new("b");
//         let one = many0(char::<_, ()>('a'));
//         let eof = terminal(eof);

//         let (mut input, tokens) = one.parse(input, &eof).unwrap();
//         assert_eq!(tokens, vec![]);

//         assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec![]);
//     }

//     #[test]
//     pub fn test_many1() {
//         let input = CollectUnrecognized::new("aba");
//         let one = many1(char::<_, ()>('a'));
//         let eof = terminal(eof);

//         let (mut input, tokens) = one.parse(input, &eof).unwrap();
//         assert_eq!(tokens, vec!['a', 'a']);

//         assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['b']);
//     }

//     #[test]
//     pub fn test_many1_empty() {
//         let input = CollectUnrecognized::new("b");
//         let one = many1(char::<_, ()>('a'));
//         let eof = terminal(eof);

//         assert!(one.parse(input, &eof).is_err());
//     }
// }

use std::iter::once;

use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult};

use super::{Input, Recognizer, RecoverableParser};

/// Parser that skips and reports unrecognized tokens up until a recovery point.
pub struct SkipUnrecognized<P>(P);

impl<I, O, E, R, P> RecoverableParser<I, O, E, R> for SkipUnrecognized<P>
where
    P: RecoverableParser<I, O, E, R>,
    R: Recognizer<I, E>,
    I: Input + Clone,
    E: ParseError<I>,
{
    fn recovery_point(&self) -> R {
        self.0.recovery_point()
    }

    fn parse(&self, mut input: I, recovery_point: R) -> IResult<I, O, E> {
        loop {
            if let Ok(result) = self.0.parse(input.clone(), recovery_point) {
                return Ok(result);
            }

            if let Ok(_) = recovery_point.recognize(input.clone()) {
                return Err(Err::Error(E::from_error_kind(input, ErrorKind::Fail)));
            }

            if let Some(token) = input.next() {
                input.unrecognized(once(token));
            }

            return Err(Err::Error(E::from_error_kind(input, ErrorKind::Fail)));
        }

        // move |input: I| -> IResult<I, O, E> {
        // This parser returns:
        // - Ok(Some(_)) if it has parsed something
        // - Ok(None)    if it has reached the recovery point
        // - Err(_)      along the way
        // let parser = self
        //     .0
        //     .parser(&recovery_point)
        //     .map(Some)
        //     .or(recovery_point.recognizer().map(|_| None));

        // let mut parser = many_till(next, parser);

        // let (mut input, (rest, value)) = parser.parse(input)?;
        // match value {
        //     Some(value) => {
        //         assert!(rest.is_empty());
        //         input.unrecognized(rest);
        //         Ok((input, value))
        //     }
        //     None => Err(Err::Error(E::from_error_kind(input, ErrorKind::Fail))),
        // }
        // }
    }
}

/// Returns a parser that can skip and report unrecognized tokens up until a
/// recovery point.
pub fn skip_unrecognized<P>(parser: P) -> SkipUnrecognized<P> {
    SkipUnrecognized(parser)
}

// #[cfg(test)]
// mod tests {
//     use nom::combinator::eof;

//     use crate::mock::{char, CollectUnrecognized};
//     use crate::{terminal, RecoverableParser};

//     use super::skip_unrecognized;

//     #[test]
//     pub fn test_skip_unrecognized() {
//         let input = CollectUnrecognized::new("123");

//         let one = char::<_, ()>('1');
//         let two = char('2');
//         let three = skip_unrecognized(char('3'));
//         let eof = terminal(eof);

//         let (input, token) = one.parse(input, &eof).unwrap();
//         assert_eq!(token, '1');

//         assert!(three.parse(input.clone(), two).is_err());

//         let (mut input, token) = three.parse(input, &eof).unwrap();
//         assert_eq!(token, '3');

//         assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['2']);
//     }
// }

use nom::error::{ErrorKind, ParseError};
use nom::multi::many_till;
use nom::{Err, IResult, Parser};

use super::next::next;
use super::{Input, Recognizer, RecoverableParser};

/// Parser that skips and reports unrecognized tokens up until a recovery point.
pub struct SkipUnrecognized<P>(P);

impl<I, E, P> Recognizer<I, E> for SkipUnrecognized<P>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O, E, P> RecoverableParser<I, O, E> for SkipUnrecognized<P>
where
    P: RecoverableParser<I, O, E>,
    I: Input + Clone,
    E: ParseError<I>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        move |input: I| -> IResult<I, O, E> {
            // This parser returns:
            // - Ok(Some(_)) if it has parsed something
            // - Ok(None)    if it has reached the recovery point
            // - Err(_)      along the way
            let parser = self
                .0
                .parser(&recovery_point)
                .map(Some)
                .or(recovery_point.recognizer().map(|_| None));

            let mut parser = many_till(next, parser);

            let (mut input, (rest, value)) = parser.parse(input)?;
            match value {
                Some(value) => {
                    input.unrecognized(rest);
                    Ok((input, value))
                }
                None => Err(Err::Error(E::from_error_kind(input, ErrorKind::Fail))),
            }
        }
    }
}

/// Returns a parser that can skip and report unrecognized tokens up until a
/// recovery point.
pub fn skip_unrecognized<P>(parser: P) -> SkipUnrecognized<P> {
    SkipUnrecognized(parser)
}

#[cfg(test)]
mod tests {
    use nom::combinator::eof;
    use nom::Parser;

    use crate::mock::{char, CollectUnrecognized};
    use crate::{terminal, RecoverableParser};

    use super::skip_unrecognized;

    #[test]
    pub fn test_skip_unrecognized() {
        let input = CollectUnrecognized::new("123");

        let one = char::<_, ()>('1');
        let two = char('2');
        let three = skip_unrecognized(char('3'));
        let eof = terminal(eof);

        let (input, token) = one.parser(&eof).parse(input).unwrap();
        assert_eq!(token, '1');

        assert!(three.parser(two).parse(input.clone()).is_err());

        let (mut input, token) = three.parser(&eof).parse(input).unwrap();
        assert_eq!(token, '3');

        assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['2']);
    }
}

use nom::error::ParseError;
use nom::Parser;

use super::skip::{skip_unrecognized, SkipUnrecognized};
use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that invokes an underlying parser multiple times and
/// returns a `Vec<T>` of results.
pub struct Many<P>(P, bool);

impl<I, E, P> Recognizer<I, E> for Many<P>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O, E, P> RecoverableParser<I, Vec<O>, E> for Many<P>
where
    I: Clone + Input,
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, Vec<O>, E>
    where
        R: Recognizer<I, E>,
    {
        move |input: I| {
            let parser = self.0.parser((&recovery_point).or(&self.0));

            match self.1 {
                false => nom::multi::many0(parser).parse(input),
                true => nom::multi::many1(parser).parse(input),
            }
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

#[cfg(test)]
mod tests {
    use nom::combinator::eof;
    use nom::Parser;

    use crate::mock::{char, CollectUnrecognized};
    use crate::{terminal, RecoverableParser};

    use super::{many0, many1};

    #[test]
    pub fn test_many0() {
        let input = CollectUnrecognized::new("aba");
        let one = many0(char::<_, ()>('a'));
        let eof = terminal(eof);

        let (mut input, tokens) = one.parser(&eof).parse(input).unwrap();
        assert_eq!(tokens, vec!['a', 'a']);

        assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['b']);
    }

    #[test]
    pub fn test_many0_empty() {
        let input = CollectUnrecognized::new("b");
        let one = many0(char::<_, ()>('a'));
        let eof = terminal(eof);

        let (mut input, tokens) = one.parser(&eof).parse(input).unwrap();
        assert_eq!(tokens, vec![]);

        assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    pub fn test_many1() {
        let input = CollectUnrecognized::new("aba");
        let one = many1(char::<_, ()>('a'));
        let eof = terminal(eof);

        let (mut input, tokens) = one.parser(&eof).parse(input).unwrap();
        assert_eq!(tokens, vec!['a', 'a']);

        assert_eq!(input.unrecognized().collect::<Vec<_>>(), vec!['b']);
    }

    #[test]
    pub fn test_many1_empty() {
        let input = CollectUnrecognized::new("b");
        let one = many1(char::<_, ()>('a'));
        let eof = terminal(eof);

        assert!(one.parser(&eof).parse(input).is_err());
    }
}

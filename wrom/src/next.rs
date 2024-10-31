use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult};

use super::Input;

pub fn next<I, E>(mut input: I) -> IResult<I, I::Item, E>
where
    I: Input,
    E: ParseError<I>,
{
    match input.next() {
        Some(token) => Ok((input, token)),
        None => Err(Err::Error(E::from_error_kind(input, ErrorKind::ManyTill))),
    }
}

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;
    use nom::{Err, Parser};

    use super::next;

    #[test]
    fn test_next() {
        let input = "123";
        let (input, item) = next::<_, ()>.parse(input).unwrap();
        assert_eq!(item, '1');

        let (input, item) = next::<_, ()>.parse(input).unwrap();
        assert_eq!(item, '2');

        let (input, item) = next::<_, ()>.parse(input).unwrap();
        assert_eq!(item, '3');

        assert!(matches!(
            next.parse(input),
            Err(Err::Error((_, ErrorKind::ManyTill)))
        ));
    }
}

use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult, Parser};

use super::{Recognizer, RecoverableParser};

/// Parser that succeeds if any of the recoverable parsers in `L` succeed.
pub struct Alt<L>(L);

impl<I, E, R, const N: usize> Recognizer<I, E> for Alt<[R; N]>
where
    I: Clone,
    E: ParseError<I>,
    R: Recognizer<I, E>,
{
    #[inline(always)]
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        for i in 0..N {
            if let Ok(result) = self.0[i].recognize(input.clone()) {
                return Ok(result);
            }
        }

        Err(Err::Error(E::from_error_kind(input, ErrorKind::Alt)))
    }
}

impl<I, O, E, P, const N: usize> RecoverableParser<I, O, E> for Alt<[P; N]>
where
    I: Clone,
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    #[inline(always)]
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        for i in 0..N {
            if let Ok(result) = self.0[i].parse(input.clone(), &recovery_point) {
                return Ok(result);
            }
        }

        Err(Err::Error(E::from_error_kind(input, ErrorKind::Alt)))
    }
}

/// Returns a recoverable parser that succeeds if any of the recoverable parsers
/// in `L` succeed.
pub fn alt<L>(list: L) -> Alt<L> {
    Alt(list)
}

macro_rules! alt {
    ($first:ident $($ident:ident)*) => {
        alt!(@ $first ; $($ident)*);
    };
    (@ $($ident:ident)* ; $next:ident $($rest:ident)*) => {
        alt!(@ $($ident)* ;);
        alt!(@ $($ident)* $next ; $($rest)*);
    };
    (@ $($ident:ident)* ;) => {
        #[allow(non_snake_case)]
        impl<I, E, $($ident),*> Recognizer<I, E> for Alt<($($ident,)*)>
        where
            I: Clone,
            E: ParseError<I>,
            $(
                $ident: Recognizer<I, E>,
            )*
        {
            #[inline(always)]
            fn recognize(&self, input: I) -> IResult<I, (), E> {
                let ($($ident,)*) = &self.0;

                nom::branch::alt((
                    $(
                        #[inline(always)]
                        |input| $ident.recognize(input),
                    )*
                )).parse(input)
            }
        }

        #[allow(non_snake_case)]
        impl<I, O, E, $($ident),*> RecoverableParser<I, O, E> for Alt<($($ident,)*)>
        where
            I: Clone,
            $(
                $ident: RecoverableParser<I, O, E>,
            )*
            E: ParseError<I>,
        {
            #[inline(always)]
            fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
            where
                R: Recognizer<I, E>,
            {
                let ($($ident,)*) = &self.0;

                nom::branch::alt((
                    $(
                        #[inline(always)]
                        |input| $ident.parse(input, &recovery_point),
                    )*
                )).parse(input)
            }
        }
    };
}

alt!(A B C D F G H J K L M N P Q S T U);

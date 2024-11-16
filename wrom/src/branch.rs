use nom::error::ParseError;
use nom::{IResult, Parser};

use super::{Recognizer, RecoverableParser};

/// Parser that succeeds if any of the recoverable parsers in `L` succeed.
pub struct Alt<L>(L);

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
        impl<I, O, E, $($ident),*, R> RecoverableParser<I, O, E, R> for Alt<($($ident,)*)>
        where
            I: Clone,
            $(
                $ident: RecoverableParser<I, O, E, R>,
            )*
            E: ParseError<I>,
            R: Recognizer<I, E>,
        {
            fn recognizer(&self) -> R {
                let ($($ident,)*) = &self.0;

                R::default()
                    $(
                        .or($ident.recognizer())
                    )*
            }

            fn parse(&self, input: I, recovery_point: R) -> IResult<I, O, E>
            {
                let ($($ident,)*) = &self.0;

                nom::branch::alt((
                    $(
                        |input| $ident.parse(input, recovery_point),
                    )*
                )).parse(input)
            }
        }
    };
}

alt!(A B C D F G H J K L M N P Q S T U);

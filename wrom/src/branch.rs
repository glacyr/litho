use nom::error::ParseError;
use nom::Parser;

use super::{Recognizer, RecoverableParser};

/// Parser that succeeds if any of the recoverable parsers in `L` succeed.
pub struct Alt<L>(L);

/// Returns a recoverable parser that succeeds if any of the recoverable parsers
/// in `L` succeed.
pub fn alt<L, I, O, E>(list: L) -> impl RecoverableParser<I, O, E>
where
    Alt<L>: RecoverableParser<I, O, E>,
{
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
            $(
                $ident: Recognizer<I, E>,
            )*
            E: ParseError<I>,
        {
            fn recognizer(&self) -> impl Parser<I, (), E> {
                let ($($ident,)*) = &self.0;

                nom::branch::alt((
                    $(
                        $ident.recognizer(),
                    )*
                ))
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
            fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
            where
                R: Recognizer<I, E>,
            {
                move |input: I| {
                    let ($($ident,)*) = &self.0;

                    nom::branch::alt((
                        $(
                            $ident.parser(&recovery_point),
                        )*
                    )).parse(input)
                }
            }
        }
    };
}

alt!(A B C D F G H J K L M N P Q S T U);

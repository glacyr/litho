use nom::error::{ErrorKind, ParseError};

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
            $(
                $ident: RecoverableParser<I, O, E, R>,
            )*
            E: for<'b> ParseError<&'b I>,
            R: Recognizer<I, E>,
        {
            #[inline(always)]
            fn recognizer(&self) -> R {
                let ($($ident,)*) = &self.0;

                R::default()
                    $(
                        .or($ident.recognizer())
                    )*
            }

            #[inline]
            fn parse(&mut self, input: &mut I, recovery_point: R) -> Result<O, E>
            {
                let ($($ident,)*) = &mut self.0;

                $(
                    if $ident.recognizer().recognize(input).is_ok() {
                        return $ident.parse(input, recovery_point);
                    }
                )*

                Err(E::from_error_kind(input, ErrorKind::Alt))
            }
        }
    };
}

alt!(A B C D F G H J K L M N P Q S T U);

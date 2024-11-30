use super::{Input, RecoverableParser};

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
    (@ $first:ident $($ident:ident)* ;) => {
        #[allow(non_snake_case)]
        impl<I, O, E, $first, $($ident,)*> RecoverableParser<I, O, E> for Alt<($first, $($ident,)*)>
        where
            I: Input,
            $first: RecoverableParser<I, O, E>,
            $(
                $ident: RecoverableParser<I, O, E>,
            )*
        {
            #[inline(always)]
            fn recognizer(&self) -> I::Recognizer {
                let ($first, $($ident,)*) = &self.0;

                $first.recognizer()
                    $(
                        | $ident.recognizer()
                    )*
            }

            #[inline(always)]
            fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<O, E>
            {
                let ($first, $($ident,)*) = &mut self.0;

                $(
                    if input.recognize($ident.recognizer()) {
                        return $ident.parse(input, recovery_point);
                    }
                )*

                $first.parse(input, recovery_point)
            }
        }
    };
}

alt!(A B C D F G H J K L M N P Q S T U);

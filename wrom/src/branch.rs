use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult};

use super::{Recognizer, RecoverableParser};

pub trait AltRecognize<I, E> {
    fn recognize(&self, input: I) -> IResult<I, (), E>;
}

pub trait AltParse<I, O, E>
where
    I: Iterator,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
    where
        R: Recognizer<I, E>;
}

pub trait AltList<I, O, E>: AltRecognize<I, E> + AltParse<I, O, E>
where
    I: Iterator,
{
}

impl<I, O, E, T> AltList<I, O, E> for T
where
    I: Iterator,
    T: AltRecognize<I, E> + AltParse<I, O, E>,
{
}

pub struct Alt<L>(L);

impl<I, E, L> Recognizer<I, E> for Alt<L>
where
    I: Iterator,
    L: AltRecognize<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E, L> RecoverableParser<I, O, E> for Alt<L>
where
    I: Iterator,
    L: AltRecognize<I, E> + AltParse<I, O, E>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        self.0.parse(input, recovery_point)
    }
}

pub fn alt<L, I, O, E>(list: L) -> impl RecoverableParser<I, O, E>
where
    L: AltList<I, O, E>,
    I: Iterator,
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
        impl<I, E, $($ident),*> AltRecognize<I, E> for ($($ident,)*)
        where
            I: Clone,
            $(
                $ident: Recognizer<I, E>,
            )*
            E: ParseError<I>,
        {
            fn recognize(&self, input: I) -> IResult<I, (), E> {
                let ($($ident,)*) = self;

                $(
                    if let Ok(ok) = $ident.recognize(input.clone()) {
                        return Ok(ok);
                    }
                )*

                return Err(Err::Error(E::from_error_kind(input, ErrorKind::Alt)));
            }
        }

        #[allow(non_snake_case)]
        impl<I, O, E, $($ident),*> AltParse<I, O, E> for ($($ident,)*)
        where
            I: Iterator + Clone,
            I::Item: Clone,
            $(
                $ident: RecoverableParser<I, O, E>,
            )*
            E: ParseError<I>,
        {
            fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, O, E>
            where
                R: Recognizer<I, E>,
            {
                let ($($ident,)*) = self;

                $(
                    if let Ok(ok) = $ident.parse(input.clone(), &recovery_point) {
                        return Ok(ok);
                    }
                )*

                return Err(Err::Error(E::from_error_kind(input, ErrorKind::Alt)));
            }
        }
    };
}

alt!(A B C D F G H J K L M N P Q S T U V W X Y Z);

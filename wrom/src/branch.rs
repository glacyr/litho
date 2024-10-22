use nom::error::ParseError;
use nom::Parser;

use super::{Recognizer, RecoverableParser};

pub trait AltRecognize<I, E> {
    fn recognizer(&self) -> impl Parser<I, (), E>;
}

pub trait AltParse<I, O, E>
where
    I: Iterator,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
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

/// Combinator that succeeds if any of the parsers in `L` succeeds.
pub struct Alt<L>(L);

impl<I, E, L> Recognizer<I, E> for Alt<L>
where
    I: Iterator,
    L: AltRecognize<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O, E, L> RecoverableParser<I, O, E> for Alt<L>
where
    I: Iterator,
    L: AltRecognize<I, E> + AltParse<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        self.0.parser(recovery_point)
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
            fn recognizer(&self) -> impl Parser<I, (), E> {
                let ($($ident,)*) = self;

                nom::branch::alt((
                    $(
                        $ident.recognizer(),
                    )*
                ))
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
            fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
            where
                R: Recognizer<I, E>,
            {
                move |input: I| {
                    let ($($ident,)*) = self;

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

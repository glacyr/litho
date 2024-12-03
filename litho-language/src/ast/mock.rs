use std::marker::PhantomData;

use arbitrary::{Arbitrary, Result, Unstructured};

use crate::lex::raw::RawToken;
use crate::lex::{Token, TokenKind};

use super::{types::*, ContextValue};

pub fn arbitrary_punctuators<'a, T>(
    lhs: &'static str,
    rhs: &'static str,
) -> (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>)
where
    T: From<&'static str>,
{
    (
        Punctuator::new(lhs.into()),
        Recoverable::Present(Punctuator::new(rhs.into())),
    )
}

pub fn arbitrary_at_least_one<'a, T>(u: &mut Unstructured<'a>) -> Result<T>
// where
//     T: Arbitrary<'a>,
{
    todo!()
    // let value = Vec::<T>::arbitrary(u)?;

    // match value.is_empty() {
    //     true => Ok(vec![T::arbitrary(u)?]),
    //     false => Ok(value),
    // }
}

pub fn arbitrary_present_at_least_one<'a, T>(u: &mut Unstructured<'a>) -> Result<Recoverable<T>>
where
    // T: Arbitrary<'a>,
{
    todo!()
    // let value = Vec::<T>::arbitrary(u)?;

    // match value.is_empty() {
    //     true => Ok(vec![T::arbitrary(u)?].into()),
    //     false => Ok(value.into()),
    // }
}

impl<'a, T> Arbitrary<'a> for Name<'a, T>
where
    T: From<&'static str>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let names = ["a", "A", "aa", "Aa", "aA", "a0", "A0", "_a", "_A", "__"];
        let name = *u.choose(&names)?;

        Ok(Name::new(name.into()))
    }
}

impl<'a, T> Arbitrary<'a> for IntValue<'a, T>
where
    T: From<&'static str>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let values = ["0", "-1", "1", "-12", "12", "-0"];
        let value = *u.choose(&values)?;

        let token = Token::from(RawToken {
            span: Default::default(),
            kind: TokenKind::IntValue,
            source: value.into(),
            marker: PhantomData,
        });

        match token {
            Token::IntValue(value) => Ok(value),
            _ => unreachable!(),
        }
    }
}

impl<'a, T> Arbitrary<'a> for FloatValue<'a, T>
where
    T: From<&'static str>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let values = ["0.0", "-1.0", "1.0", "-1.2", "1.2", "-0.0"];
        let value = *u.choose(&values)?;

        let token = Token::from(RawToken {
            span: Default::default(),
            kind: TokenKind::FloatValue,
            source: value.into(),
            marker: PhantomData,
        });

        match token {
            Token::FloatValue(value) => Ok(value),
            _ => unreachable!(),
        }
    }
}

impl<'a, T> Arbitrary<'a> for StringValue<'a, T>
where
    T: From<&'static str>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let values = ["\"hello\""];
        let value = *u.choose(&values)?;

        let token = Token::from(RawToken {
            span: Default::default(),
            kind: TokenKind::StringValue,
            source: value.into(),
            marker: PhantomData,
        });

        match token {
            Token::StringValue(value) => Ok(value),
            _ => unreachable!(),
        }
    }
}

impl<'a, 'arbitrary, T> Arbitrary<'arbitrary> for NonNullType<'a, T>
where
    T: ContextValue<'a> + From<&'static str>,
{
    fn arbitrary(u: &mut Unstructured<'arbitrary>) -> Result<Self> {
        todo!()
        // let ty = Type::arbitrary(u)?;

        // Ok(match ty {
        //     Type::NonNull(ty) => ty,
        //     _ => NonNullType {
        //         ty: ty.into(),
        //         bang: Punctuator::new("!".into()),
        //     },
        // })
    }
}

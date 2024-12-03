use std::marker::PhantomData;

use wrom::{Input, RecoverableParser};

use super::{Context, ContextValue, Shared};

pub struct IntoShared<P, O>(P, PhantomData<O>);

impl<'a, I, O, E, P, T> RecoverableParser<I, Shared<'a, T, O>, E> for IntoShared<P, O>
where
    P: RecoverableParser<I, O, E>,
    I: Input + Context<'a, T>,
    O: 'a,
    T: ContextValue<'a>,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(
        &mut self,
        input: &mut I,
        recovery_point: I::Recognizer,
    ) -> Result<Shared<'a, T, O>, E> {
        let value = self.0.parse(input, recovery_point)?;
        Ok(input.shared(value))
    }
}

pub struct MapInput<P, F, O>(P, F, PhantomData<O>);

impl<I, O, O2, E, P, F> RecoverableParser<I, O2, E> for MapInput<P, F, O>
where
    I: Input,
    P: RecoverableParser<I, O, E>,
    F: FnMut(&mut I, O) -> O2,
{
    #[inline(always)]
    fn recognizer(&self) -> I::Recognizer {
        self.0.recognizer()
    }

    #[inline(always)]
    fn parse(&mut self, input: &mut I, recovery_point: I::Recognizer) -> Result<O2, E> {
        let value = self.0.parse(input, recovery_point)?;
        Ok(self.1(input, value))
    }
}

pub trait RecoverableParserExt<I, O, E>: RecoverableParser<I, O, E>
where
    I: Input,
{
    #[inline(always)]
    fn into_shared(self) -> IntoShared<Self, O>
    where
        Self: Sized,
    {
        IntoShared(self, PhantomData)
    }

    fn map_input<F>(self, f: F) -> MapInput<Self, F, O>
    where
        Self: Sized,
    {
        MapInput(self, f, PhantomData)
    }
}

impl<I, O, E, P> RecoverableParserExt<I, O, E> for P
where
    P: RecoverableParser<I, O, E>,
    I: Input,
{
}

use nom::InputLength;

use super::Input;

#[derive(Clone)]
pub struct CollectUnrecognized<I>
where
    I: Input,
{
    input: I,
    unrecognized: Vec<I::Item>,
}

impl<I> CollectUnrecognized<I>
where
    I: Input,
{
    pub fn new(input: I) -> CollectUnrecognized<I> {
        CollectUnrecognized {
            input,
            unrecognized: Default::default(),
        }
    }

    pub fn into_inner(self) -> I {
        self.input
    }

    pub fn unrecognized(&mut self) -> impl Iterator<Item = I::Item> + '_ {
        self.unrecognized.drain(..)
    }
}

impl<I> InputLength for CollectUnrecognized<I>
where
    I: Input,
{
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<I> Input for CollectUnrecognized<I>
where
    I: Input + Clone,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next()
    }

    fn unrecognized(&mut self, item: I::Item) {
        self.unrecognized.push(item);
    }
}

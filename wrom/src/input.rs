use nom::InputLength;

use super::Missing;

pub trait Input:
    Clone + Iterator<Item = <Self as Input>::Item> + InputLength + Extend<<Self as Input>::Item>
{
    type Item;
    type Missing: Default + Missing;

    fn missing(&self, missing: Self::Missing) -> <Self::Missing as Missing>::Error;
}

// #[derive(Clone, Debug)]
// pub struct Recover<I, F, M>
// where
//     I: Iterator,
//     F: Fn(&I, &'static str) -> M,
// {
//     input: I,
//     missing: F,
//     unrecognized: Vec<I::Item>,
// }

// impl<I, F, M> Recover<I, F, M>
// where
//     I: Iterator,
//     F: Fn(&'static str) -> M,
// {
//     pub fn new(input: I, missing: F) -> Recover<I, F, M> {
//         Recover {
//             input,
//             missing,
//             unrecognized: vec![],
//         }
//     }

//     pub fn collect_all<U>(self) -> U
//     where
//         U: FromIterator<I::Item>,
//     {
//         self.input.chain(self.unrecognized).collect()
//     }

//     pub fn finish(self) -> (I, Vec<I::Item>) {
//         (self.input, self.unrecognized)
//     }
// }

// impl<I, F, M> Iterator for Recover<I, F, M>
// where
//     I: Iterator,
//     F: Fn(&I, &'static str) -> M,
// {
//     type Item = I::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.input.next()
//     }
// }

// impl<I, F, M> Extend<I::Item> for Recover<I, F, M>
// where
//     I: Iterator,
//     F: Fn(&I, &'static str) -> M,
// {
//     fn extend<T: IntoIterator<Item = I::Item>>(&mut self, iter: T) {
//         self.unrecognized.extend(iter)
//     }
// }

// impl<I, F, M> InputLength for Recover<I, F, M>
// where
//     I: Iterator + InputLength,
//     F: Fn(&I, &'static str) -> M,
// {
//     fn input_len(&self) -> usize {
//         self.input.input_len()
//     }
// }

// impl<I, F, M> Input for Recover<I, F, M>
// where
//     I: Clone + Iterator + InputLength,
//     I::Item: Clone,
//     F: Fn(&I, &'static str) -> M + Clone,
//     M: Clone,
// {
//     type Item = I::Item;
//     type Missing = M;

//     fn missing(&self, missing: &'static str) -> Self::Missing {
//         (self.missing)(&self.input, missing)
//     }
// }

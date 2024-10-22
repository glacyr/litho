use nom::InputLength;

use super::Missing;

/// Implemented by types that can be read from.
pub trait Input:
    Clone + Iterator<Item = <Self as Input>::Item> + InputLength + Extend<<Self as Input>::Item>
{
    /// Type of a single input item.
    type Item;
    type Missing: Default + Missing;

    fn missing(&self, missing: Self::Missing) -> <Self::Missing as Missing>::Error;
}

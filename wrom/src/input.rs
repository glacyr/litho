use std::iter::Peekable;
use std::str::Chars;

/// Implemented by types that can be read from.
pub trait Input: Sized {
    /// Type of item that this input's [`Input::next`] returns.
    type Item;

    /// Should return the next item from this input.
    fn next(&mut self) -> Option<Self::Item>;

    /// Should return a peek at the next item from this input. Calling this
    /// repeatedly without calling `Input::next` should return the same item.
    fn peek(&mut self) -> Option<&Self::Item>;

    /// Called by parsers when one or more tokens were unrecognized.
    /// Implementors of this trait can collect these items and turn them into
    /// diagnostics.
    fn unrecognized(&mut self, item: Self::Item);
}

impl<'a> Input for Peekable<Chars<'a>> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        Iterator::next(self)
    }

    fn peek(&mut self) -> Option<&Self::Item> {
        Peekable::peek(self)
    }

    fn unrecognized(&mut self, _item: Self::Item) {
        // Discard unrecognized tokens.
    }
}

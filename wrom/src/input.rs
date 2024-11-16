use nom::InputLength;

/// Implemented by types that can be read from.
pub trait Input: Clone + InputLength + Sized {
    /// Type of item that this input's [`Input::next`] returns.
    type Item;

    /// Should return the next item from this input.
    fn next(&mut self) -> Option<Self::Item>;

    /// Called by parsers when one or more tokens were unrecognized.
    /// Implementors of this trait can collect these items and turn them into
    /// diagnostics.
    fn unrecognized(&mut self, item: Self::Item);
}

impl<'a> Input for &'a str {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        let f = self.as_bytes()[0] as char;
        *self = &self[1..];
        Some(f)
    }

    fn unrecognized(&mut self, _item: Self::Item) {
        // Discard unrecognized tokens.
    }
}

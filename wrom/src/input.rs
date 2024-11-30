use std::ops::BitOr;

/// Implemented by types that can be read from.
pub trait Input: Sized {
    /// Type that can be used to recognize the start of a grammar from an input.
    type Recognizer: Copy + Default + BitOr<Output = Self::Recognizer>;

    /// Should return a boolean that indicates whether the recognizer is
    /// successful.
    fn recognize(&mut self, recognizer: Self::Recognizer) -> bool;

    /// Called by parsers when one or more tokens were unrecognized.
    /// Implementors of this trait can collect these items and turn them into
    /// diagnostics.
    fn discard(&mut self, recognizer: Self::Recognizer, recovery_point: Self::Recognizer) -> bool;
}

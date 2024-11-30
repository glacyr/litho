/// Implemented by types (usually parsers) that can recognize something from an
/// input, without consuming it. This is the starting point of your recoverable
/// parser.
pub trait Recognizer<I, E>: Default + Copy {
    /// Returns a parser that should return `Ok(_)` when it recognizes the start
    /// of a parsing rule and `Err(_)` otherwise, without consuming its input.
    fn recognize(self, input: &mut I) -> Result<(), E>;

    /// Returns a recognizer that succeeds when either `self` or the `other`
    /// recognizer succeeds.
    fn or(self, other: Self) -> Self;
}

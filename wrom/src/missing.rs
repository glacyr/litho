/// Trait implemented by types that can be missing.
///
/// Implementors of this trait should be agnostic to the input and its location.
/// For example, a `RightParenthesis` could be [`Missing`], and
/// [`Missing::Error`] would then contain `RightParenthesis` and the span that
/// caused the error.
pub trait Missing<I> {
    /// Error that is returned by recoverable parsers when this type is missing
    /// and the parser recovered.
    type Error;

    /// Called by parsers when something (like a token or a sequence of tokens)
    /// is expected but missing.
    fn error(&self, input: &mut I) -> Self::Error;
}

impl<I> Missing<I> for () {
    type Error = ();

    fn error(&self, _input: &mut I) -> Self::Error {
        ()
    }
}

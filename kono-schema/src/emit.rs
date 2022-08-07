/// Implementations of this trait turn a high level Rust-like representation of
/// a GraphQL type into an actual `graphql_parser::schema::*` type.
pub trait Emit {
    /// Type of target an implementation of this trait emits.
    type Target;

    /// Emits a low-level GraphQL type based on the high-level Rust-like type
    /// that implements this trait.
    fn emit(self) -> Self::Target;
}

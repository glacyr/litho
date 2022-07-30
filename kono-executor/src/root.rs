/// Should be implemented by values that can represent the root of a GraphQL
/// request (i.e. `Query`, `Mutation` or `Subscription`).
pub trait Root {
    /// Should return a value that represents the `Query` type.
    fn query() -> Self;

    /// Should return a value that represents the `Mutation` type.
    fn mutation() -> Self;

    /// Should return a value that represents the `Subscription` type.
    fn subscription() -> Self;
}

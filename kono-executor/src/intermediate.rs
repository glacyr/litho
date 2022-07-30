use super::Value;

/// Represents a resolver's result (if successful).
#[derive(Debug, PartialEq, Eq)]
pub enum Intermediate<T> {
    /// Collection of intermediate values (used when a GraphQL selection set
    /// should be applied to an array).
    Collection(Vec<Intermediate<T>>),

    /// Represents a single unresolved object.
    Object(T),

    /// Represents a resolved value.
    Value(Value),
}

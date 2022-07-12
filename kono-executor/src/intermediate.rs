use super::Value;

/// Represents a resolver's result (if successful).
#[derive(Debug, PartialEq, Eq)]
pub enum Intermediate<T> {
    Collection(Vec<Intermediate<T>>),
    Object(T),
    Value(Value),
}

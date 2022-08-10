#![deny(missing_docs)]

//! `kono-schema` contains tools that make it easy to represent high-level Rust
//! types such as enums with fields in GraphQL, which doesn't natively support
//! such types.
//!
//! ### Extensions
//! `kono-schema` implements several extensions that are not natively supported in GraphQL.
//!
//! - __Types.__ Types
//! [without fields](./struct.ItemType.html#types-without-fields), with
//! [named fields](./struct.ItemType.html#types-with-named-fields) and with
//! [unnamed fields](./struct.ItemType.html#types-with-unnamed-fields).
//! - __Enums.__ Enums with [named fields](./struct.ItemEnum.html#enums-with-named-fields)
//! and [unnamed fields](./struct.ItemEnum.html#enums-with-unnamed-fields).
//! - __Unions.__ Unions with [scalars](./struct.ItemEnum.html#unions-with-scalars).

mod directives;
mod emit;
mod enums;
mod fields;
mod inputs;
mod join;
mod scalars;
mod ty;
mod types;

pub use directives::Directive;
pub use emit::Emit;
pub use enums::{ItemEnum, Variant};
pub use fields::{Field, Fields};
use graphql_parser::schema;
pub use inputs::{InputValue, ItemInput};
pub use scalars::ItemScalar;
pub use ty::Type;
pub use types::ItemType;

/// Trait implemented by types that have a schema.
pub trait Schema {
    /// Should return the schema of the type that implements this trait.
    fn schema(&self) -> Vec<Item>;
}

/// Single item in a high-level Rust-like GraphQL scheme (including enums with
/// fields, types without fields, union inputs, etc).
pub enum Item {
    /// Rust-like enum, optionally with named or unnamed fields.
    Enum(ItemEnum),

    /// Rust-like `Deserialize`able struct.
    Input(ItemInput),

    /// Scalar value (either built-in, like `String`s and `Int`s, or custom).
    Scalar(ItemScalar),

    /// Rust-like `Serialize`able struct.
    Type(ItemType),
}

impl Emit for Item {
    type Target = Vec<schema::TypeDefinition<'static, String>>;

    fn emit(self) -> Self::Target {
        match self {
            Item::Enum(item) => item.emit(),
            Item::Input(item) => item.emit(),
            Item::Scalar(item) => item.emit(),
            Item::Type(item) => item.emit(),
        }
    }
}

impl Emit for Vec<Item> {
    type Target = schema::Document<'static, String>;

    fn emit(self) -> Self::Target {
        schema::Document {
            definitions: self
                .into_iter()
                .flat_map(Emit::emit)
                .map(schema::Definition::TypeDefinition)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use graphql_parser::schema;
    use unindent::unindent;

    use super::Emit;

    pub fn test_eq<T>(definition: T, expected: &str)
    where
        T: Emit<Target = Vec<schema::TypeDefinition<'static, String>>>,
    {
        let document = schema::Document {
            definitions: definition
                .emit()
                .into_iter()
                .map(schema::Definition::TypeDefinition)
                .collect(),
        };

        assert_eq!(
            document.format(&Default::default()).trim(),
            unindent(expected).trim()
        );
    }
}

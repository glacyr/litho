//! Data structures to represent a GraphQL Document.
//!
//! This module implements the data structures described in [Sec.
//! 2.2-2.12](https://spec.graphql.org/June2018/#sec-Language.Document) of the
//! GraphQL spec.

mod node;
mod tokens;
mod types;
mod visit;

pub use node::Node;
use node::{node, node_arc, node_enum, node_unit};
pub use types::*;
pub use visit::Visit;

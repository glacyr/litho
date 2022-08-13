mod module;
pub mod server;

pub use module::Module;
pub use server::{serve, server};

pub use kono_aspect::{AspectExt, Connection, Edge, Pagination};
pub use kono_executor::execute;
pub use kono_macros::{kono, Kono};
pub use kono_schema::Schema;

#[doc(inline)]
pub use kono_aspect as aspect;

#[doc(inline)]
pub use kono_executor as executor;

#[doc(inline)]
pub use kono_schema as schema;

#[cfg(test)]
mod tests;

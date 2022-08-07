pub mod server;

pub use server::{serve, server};

pub use kono_aspect::AspectExt;
pub use kono_macros::{kono, Kono};

#[doc(inline)]
pub use kono_aspect as aspect;

#[doc(inline)]
pub use kono_executor as executor;

#[doc(inline)]
pub use kono_schema as schema;

#[cfg(test)]
mod tests;

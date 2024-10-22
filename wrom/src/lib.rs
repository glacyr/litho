#![warn(missing_docs)]

pub mod branch;
pub mod combinator;
mod input;
mod missing;
pub mod multi;
mod parser;
mod recognizer;
mod recoverable;
mod recursive;
pub mod sequence;

pub use input::Input;
pub use missing::Missing;
pub use parser::RecoverableParser;
pub use recognizer::{terminal, Or, Recognizer};
pub use recoverable::Recoverable;
pub use recursive::recursive;

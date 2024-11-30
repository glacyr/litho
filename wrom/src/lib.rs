#![warn(missing_docs)]

//! Building blocks for creating recoverable recursive parsers with Nom.
//!
//! 1. Parsers can signal their recovery point to preceding parsers using
//!    [`recognizer`](`Recognizer::recognizer`)s.
//! 2. Parsers can [`recover`](`RecoverableParser::recover`) when expected items
//!    are missing.
//! 3. Parsers can [`skip_unrecognized`] tokens when necessary.
//! 4. [Recursive parsers](`recursive()`) with a max. depth to avoid stack overflows.

// mod boxed;
mod branch;
mod combinator;
mod input;
mod missing;
mod multi;
mod parser;
mod recognizer;
mod recoverable;
mod recursive;
mod sequence;

pub use branch::{alt, Alt};
pub use combinator::{opt, Opt};
pub use input::Input;
pub use missing::Missing;
pub use multi::many;
pub use parser::RecoverableParser;
pub use recognizer::Recognizer;
pub use recoverable::Recoverable;
pub use recursive::recursive;
pub use sequence::delimited;

#[doc(hidden)]
pub mod mock;

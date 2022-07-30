#![deny(missing_docs)]

//! `kono-executor` contains a simple low-level implementation of an
//! unopinionated async GraphQL executor. The executor does not come with
//! introspection, validation or a server.
//!
//! # Sample
//! ```rust
//! impl Error for MyError { ... }
//! impl Resolver for MyResolver { ... }
//! impl Root for MyValue { ... }
//! impl Typename for MyValue { ... }
//!
//! let executor = Executor::new(MyResolver { ... });
//! let response = executor.execute_request(..., MyContext { ... }).await?;
//! ```
//!
//! ### Advantages
//! - [x] **Asynchronous execution.**
//! - [x] **Code isolation.**
//!
//! ### Code Isolation
//! Multiple resolvers can be joined through [`impl Resolver for (A, B,
//! ...)`](./trait.Resolver.html#foreign-impls) in a way that lets you connect
//! multiple resolvers to the same executor.
//!
//! ### Roadmap
//! - [ ] Parallel execution is not yet supported.
//! - [ ] Fragments on interface and union types are not yet supported.
//! - [ ] Fragments (like `@skip`) are not yet supported.

mod error;
mod executor;
mod intermediate;
mod join;
mod resolver;
mod root;
mod typename;

pub use error::Error;
pub use executor::Executor;
pub use intermediate::Intermediate;
pub use resolver::Resolver;
pub use root::Root;
pub use typename::Typename;

pub use serde_json::Value;

//! Comprehensive error handling library with detailed backtrace tracking.
//!
//! `error2` provides enhanced error handling capabilities for Rust applications,
//! focusing on detailed error propagation tracking and ergonomic error conversion.
//!
//! # Features
//!
//! - 🔍 **Backtrace Tracking** - Automatically capture error creation location; manually record propagation with `.attach()`
//! - 🔗 **Error Chaining** - Chain errors from different libraries while preserving context
//! - 🎯 **Derive Macro** - `#[derive(Error2)]` for easy error type creation
//! - 🔄 **Type Conversion** - `Result<T, E1> -> Result<T, E2>`, `Option<T> -> Result<T, E>` with `.context()`
//! - 📦 **Type Erasure** - `BoxedError2` for anyhow-like ergonomics
//!
//! # Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! error2 = "0.13"
//! ```
//!
//! Define your error types:
//!
//! ```
//! use std::io;
//!
//! use error2::prelude::*;
//!
//! #[derive(Debug, Error2)]
//! pub enum MyError {
//!     #[error2(display("IO error: {source}"))]
//!     Io {
//!         source: io::Error,
//!         backtrace: Backtrace,
//!     },
//!
//!     #[error2(display("not found: {key}"))]
//!     NotFound { key: String, backtrace: Backtrace },
//! }
//! ```
//!
//! Use in your functions:
//!
//! ```
//! # use error2::prelude::*;
//! # use std::io;
//! # #[derive(Debug, Error2)]
//! # pub enum MyError {
//! #     #[error2(display("IO error"))]
//! #     Io { source: io::Error, backtrace: Backtrace },
//! #     #[error2(display("not found: {key}"))]
//! #     NotFound { key: String, backtrace: Backtrace },
//! # }
//! fn read_config(path: &str) -> Result<String, MyError> {
//!     // Convert io::Error to MyError::Io
//!     let content = std::fs::read_to_string(path).context(Io2)?;
//!
//!     // Convert Option to Result
//!     let value = content
//!         .lines()
//!         .next()
//!         .context(NotFound2 { key: "first line" })?;
//!
//!     Ok(value.to_string())
//! }
//! ```
//!
//! # Three Error Patterns
//!
//! Error2 supports three types of errors based on their field structure:
//!
//! ## 1. Root Error (New Error Origin)
//!
//! Use when creating a new error (not wrapping another):
//!
//! ```
//! # use error2::prelude::*;
//! #[derive(Debug, Error2)]
//! pub enum AppError {
//!     #[error2(display("invalid ID: {id}"))]
//!     InvalidId {
//!         id: i64,
//!         backtrace: Backtrace, // Only backtrace, no source
//!     },
//! }
//! ```
//!
//! ## 2. Std Error (Wrapping std::error::Error)
//!
//! Use when wrapping standard library or third-party errors:
//!
//! ```
//! # use error2::prelude::*;
//! # use std::io;
//! #[derive(Debug, Error2)]
//! pub enum AppError {
//!     #[error2(display("file error"))]
//!     FileError {
//!         source: io::Error,    // Wrapped error
//!         backtrace: Backtrace, // New backtrace
//!     },
//! }
//! ```
//!
//! ## 3. Error2 Error (Chaining Error2 Types)
//!
//! Use when wrapping another Error2 type (reuses backtrace):
//!
//! ```
//! # use error2::prelude::*;
//! # #[derive(Debug, Error2)]
//! # #[error2(display("config error"))]
//! # pub struct ConfigError { backtrace: Backtrace }
//! #[derive(Debug, Error2)]
//! pub enum AppError {
//!     #[error2(display("configuration failed"))]
//!     Config {
//!         source: ConfigError, // Only source, backtrace reused
//!     },
//! }
//! ```
//!
//! # Core Traits
//!
//! - [`Error2`] - Extends `std::error::Error` with backtrace support
//! - [`Context`] - Type conversion: `Result<T, Source> -> Result<T, Target>`, `Option<T> -> Result<T, E>`
//! - [`Attach`] - Record error propagation locations
//! - [`RootError`] - Convenience methods for creating root errors
//!
//! # Type Erasure
//!
//! [`BoxedError2`] provides anyhow-like ergonomics:
//!
//! ```
//! use error2::prelude::*;
//!
//! fn do_something() -> Result<(), BoxedError2> {
//!     std::fs::read_to_string("file.txt").context(ViaStd)?; // Convert to BoxedError2
//!     Ok(())
//! }
//! ```
//!
//! # Location Tracking
//!
//! Use `.attach()` to record error propagation:
//!
//! ```
//! # use error2::prelude::*;
//! # use std::io;
//! # #[derive(Debug, Error2)]
//! # #[error2(display("error"))]
//! # struct MyError { source: io::Error, backtrace: Backtrace }
//! # fn inner() -> Result<(), MyError> { Ok(()) }
//! fn outer() -> Result<(), MyError> {
//!     let result = inner().attach()?; // Records this location
//!     Ok(result)
//! }
//! ```
//!
//! The backtrace shows multiple locations:
//!
//! ```
//! # use error2::prelude::*;
//! # use std::io;
//! # #[derive(Debug, Error2)]
//! # #[error2(display("error"))]
//! # struct MyError { source: io::Error, backtrace: Backtrace }
//! # fn inner() -> Result<(), MyError> {
//! #     let err = io::Error::new(io::ErrorKind::NotFound, "not found");
//! #     Err(err).context(MyError2)
//! # }
//! # fn outer() -> Result<(), MyError> { inner().attach() }
//! # fn main() {
//! use regex::Regex;
//!
//! if let Err(e) = outer() {
//!     let msg = e.backtrace().error_message();
//!
//!     // Full error format with multiple locations:
//!     // MyError: error
//!     //     at /path/to/file.rs:496:14
//!     //     at /path/to/file.rs:498:45
//!     // std::io::error::Error: not found
//!
//!     let re = Regex::new(concat!(
//!         r"(?s)^.+MyError: error",
//!         r"\n    at .+\.rs:\d+:\d+",
//!         r"\n    at .+\.rs:\d+:\d+",
//!         r"\nstd::io::error::Error: not found$",
//!     ))
//!     .unwrap();
//!     assert!(re.is_match(msg.as_ref()));
//! }
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

mod _attach;
mod backtrace;
mod boxed;
mod context;
mod error2;
mod extract;
mod location;
mod macros;
mod root_error;
mod str_id;

/// Attach adapters for iterators, futures, and streams.
pub mod attach;
/// Error kind enum for downcasting [`BoxedError2`].
///
/// See [`ErrorKind`](kind::ErrorKind) for details.
pub mod kind;
/// Internal transformation traits (not for direct use).
pub mod transform;

/// Re-exports of commonly used types and traits.
///
/// Import with `use error2::prelude::*;` to get:
/// - [`Error2`] trait
/// - [`Context`], [`Attach`], [`RootError`] traits
/// - [`Backtrace`], [`BoxedError2`] types
/// - [`ViaRoot`], [`ViaStd`], [`ViaErr2`] wrappers
/// - `#[derive(Error2)]` macro (if `derive` feature enabled)
pub mod prelude {
    #[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
    #[cfg(feature = "derive")]
    pub use ::error2_derive::Error2;

    // traits
    pub use crate::{Attach as _, Context as _, RootError as _, error2::Error2};
    // types
    pub use crate::{Backtrace, BoxedError2, ViaErr2, ViaRoot, ViaStd};
}

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use ::error2_derive::Error2;

pub use self::{
    _attach::Attach,
    backtrace::Backtrace,
    boxed::{BoxedError2, ViaErr2, ViaRoot, ViaStd},
    context::Context,
    error2::Error2,
    location::Location,
    root_error::RootError,
};
pub(crate) use self::{backtrace::BakctraceEntry, extract::extract_error_message, str_id::StrId};

pub(crate) mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaPartial {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaFull {}
}

#[doc(hidden)]
pub fn push_error<E: Error2 + ?Sized>(error: &mut E, location: Location) {
    let display = error.to_string();
    let backtrace = error.backtrace_mut();
    let type_name = core::any::type_name::<E>();

    backtrace.push_error(type_name, display, location);
}

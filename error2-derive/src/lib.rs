//! Derive macro for the `error2` crate.
//!
//! This crate provides the `#[derive(Error2)]` procedural macro for automatically
//! implementing error types with backtrace support.
//!
//! # Usage
//!
//! ```
//! use error2::prelude::*;
//!
//! #[derive(Debug, Error2)]
//! pub enum MyError {
//!     #[error2(display("IO error"))]
//!     Io {
//!         source: std::io::Error,
//!         backtrace: Backtrace,
//!     },
//! }
//! ```
//!
//! See the main `error2` crate documentation for complete usage information.

mod error2;
mod generics;
mod messages;
mod parser;
mod types;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives the `Error2` trait for an error type.
///
/// This macro automatically implements:
/// - `std::error::Error`
/// - `error2::Error2`
/// - `Display` (only if `#[error2(display(...))]` is specified)
///
/// It also generates helper structs (named `{Type}2` or `{Variant}2`) for type conversion.
///
/// # Attributes
///
/// ## Type-Level Attributes
///
/// Applied to the struct or enum definition:
///
/// ### `display`
///
/// Specifies the display format for the error message. Only applicable to structs.
/// If omitted, no `Display` implementation is generated, allowing custom implementation.
///
/// ```
/// # use error2::prelude::*;
/// #[derive(Debug, Error2)]
/// #[error2(display("Failed to process: {item}"))]
/// struct ProcessError {
///     item: String,
///     backtrace: Backtrace,
/// }
/// ```
///
/// ### `vis`
///
/// Controls the visibility of generated helper structs and their fields.
/// Default is inherited from the error type.
///
/// ```
/// # use error2::prelude::*;
/// #[derive(Debug, Error2)]
/// #[error2(vis(pub(crate)), display("error: {code}"))]
/// pub struct MyError {
///     code: i32,
///     backtrace: Backtrace,
/// }
/// // Generates:
/// // pub(crate) struct MyError2<T: Into<i32>> {
/// //     pub(crate) code: T
/// // }
/// ```
///
/// ### `module`
///
/// Puts generated code into a submodule with the same name as the type (in snake_case).
///
/// ```
/// # use error2::prelude::*;
/// # mod test {
/// # use error2::prelude::*;
/// #[derive(Debug, Error2)]
/// #[error2(module, display("my error"))]
/// pub struct MyError {
///     backtrace: Backtrace,
/// }
///
/// #[derive(Debug, Error2)]
/// #[error2(module, display("read config error"))]
/// pub struct ReadConfigError {
///     backtrace: Backtrace,
/// }
///
/// // Generates:
/// // mod my_error { pub(super) struct MyError2; }
/// // mod read_config_error { pub(super) struct ReadConfigError2; }
///
/// // Usage - helper structs are in their respective modules:
/// # fn test() -> Result<(), MyError> {
/// my_error::MyError2.fail()?;
/// # Ok(())
/// # }
/// #
/// # fn test2() -> Result<(), ReadConfigError> {
/// read_config_error::ReadConfigError2.fail()?;
/// # Ok(())
/// # }
/// # }
/// ```
///
/// ## Variant-Level Attributes
///
/// Applied to enum variants:
///
/// ### `display`
///
/// Specifies the display format for this variant. If omitted, no `Display`
/// implementation is generated for this variant.
///
/// ```
/// # use error2::prelude::*;
/// #[derive(Debug, Error2)]
/// pub enum AppError {
///     #[error2(display("IO error at {path}"))]
///     Io {
///         path: String,
///         source: std::io::Error,
///         backtrace: Backtrace,
///     },
///
///     #[error2(display("Not found: {item}"))]
///     NotFound { item: String, backtrace: Backtrace },
/// }
/// ```
///
/// # Generated Helper Structs
///
/// The macro generates helper structs for type conversion, named by appending `2`:
///
/// **For structs:**
/// ```
/// # use error2::prelude::*;
/// # use std::fmt;
/// #[derive(Debug, Error2)]
/// #[error2(display("my error"))]
/// struct MyError {
///     backtrace: Backtrace,
/// }
///
/// // Generates: struct MyError2;
/// ```
///
/// **For enum variants:**
/// ```
/// # use error2::prelude::*;
/// #[derive(Debug, Error2)]
/// enum AppError {
///     #[error2(display("file error"))]
///     FileError {
///         source: std::io::Error,
///         backtrace: Backtrace,
///     },
/// }
/// // Generates: struct FileError2;
/// ```
///
/// These helper structs contain only the non-`source` and non-`backtrace` fields.
/// **All fields are generic with `Into` trait bounds**, allowing automatic type conversion:
///
/// ```
/// # use error2::prelude::*;
/// #[derive(Debug, Error2)]
/// #[error2(display("read error: {path}"))]
/// struct ReadError {
///     path: String,
///     source: std::io::Error,
///     backtrace: Backtrace,
/// }
/// // Generates (assuming ReadError has inherited visibility):
/// // struct ReadError2<T: Into<String>> { path: T }
///
/// # fn example() -> Result<(), ReadError> {
/// // No need to call .into() - automatic conversion:
/// std::fs::read_to_string("file.txt").context(ReadError2 { path: "file.txt" })?; // &str -> String
/// //
/// # Ok(())
/// # }
/// ```
///
/// For expensive conversions, use `.with_context()` for lazy evaluation:
///
/// ```
/// # use error2::prelude::*;
/// # use std::path::Path;
/// # #[derive(Debug, Error2)]
/// # #[error2(display("read error: {path}"))]
/// # struct ReadError {
/// #     path: String,
/// #     source: std::io::Error,
/// #     backtrace: Backtrace,
/// # }
/// # fn example(path: &Path) -> Result<(), ReadError> {
/// // Only converts on error:
/// std::fs::read(path).with_context(|| ReadError2 {
///     path: path.display().to_string(),
/// })?;
/// # Ok(())
/// # }
/// ```
///
/// # Boxing Large Source Errors
///
/// To avoid large `Result<T, E>` types, you can wrap the source error in a `Box` or other wrapper:
///
/// ```
/// # use error2::prelude::*;
/// use std::io;
///
/// #[derive(Debug, Error2)]
/// #[error2(display("IO error: {path}"))]
/// struct IoError {
///     path: String,
///     // Box the source to keep Result size small
///     source: Box<io::Error>,
///     backtrace: Backtrace,
/// }
///
/// # fn example(path: &str) -> Result<(), IoError> {
/// // io::Error is automatically boxed via Into trait
/// std::fs::read_to_string(path).context(IoError2 { path })?;
/// # Ok(())
/// # }
/// ```
///
/// This works because the helper struct implements `Into<Box<E>>` for `E`.
/// Any wrapper type that implements `E: Into<Wrapper<E>>` can be used.
///
/// # Display Implementation
///
/// **Important:** The `Display` trait is only implemented when `display` attribute is present.
/// This allows you to provide custom `Display` implementations:
///
/// ```
/// # use error2::prelude::*;
/// use std::fmt;
///
/// #[derive(Debug, Error2)]
/// struct MyError {
///     code: i32,
///     backtrace: Backtrace,
/// }
///
/// // Custom Display implementation (no display attribute needed)
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "Error code: {:#x}", self.code)
///     }
/// }
/// ```
#[proc_macro_derive(Error2, attributes(error2))]
pub fn error2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    error2::generate(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

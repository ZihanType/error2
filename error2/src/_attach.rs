use crate::{Error2, Location};

/// Record error propagation locations for detailed backtraces.
///
/// The `Attach` trait provides the `.attach()` method which records where
/// an error was propagated through your code, creating a complete call chain.
///
/// # Basic Usage
///
/// Use `.attach()` when propagating errors with `?`:
///
/// ```
/// use std::io;
///
/// use error2::prelude::*;
///
/// #[derive(Debug, Error2)]
/// #[error2(display("io error"))]
/// struct IoError {
///     source: io::Error,
///     backtrace: Backtrace,
/// }
///
/// fn inner() -> Result<String, IoError> {
///     std::fs::read_to_string("file.txt").context(IoError2)
/// }
///
/// fn outer() -> Result<String, IoError> {
///     let result = inner().attach()?; // Records this location in backtrace
///     Ok(result)
/// }
/// ```
///
/// # Why Use Attach?
///
/// Without `.attach()`, the backtrace only shows where the error was created.
/// With `.attach()`, it shows every point the error passed through.
///
/// ```
/// # use error2::prelude::*;
/// # use std::io;
/// # #[derive(Debug, Error2)]
/// # #[error2(display("io error"))]
/// # struct IoError { source: io::Error, backtrace: Backtrace }
/// # fn inner() -> Result<String, IoError> {
/// #     let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
/// #     Err(err).context(IoError2)
/// # }
/// # fn outer() -> Result<String, IoError> {
/// #     inner().attach()
/// # }
/// use regex::Regex;
///
/// if let Err(e) = outer() {
///     let msg = e.backtrace().error_message();
///
///     // Full error format with multiple locations:
///     // IoError: io error
///     //     at /path/to/file.rs:39:14
///     //     at /path/to/file.rs:42:13
///     // std::io::error::Error: file not found
///
///     let re = Regex::new(concat!(
///         r"(?s)^.+IoError: io error",
///         r"\n    at .+\.rs:\d+:\d+",
///         r"\n    at .+\.rs:\d+:\d+",
///         r"\nstd::io::error::Error: file not found$",
///     ))
///     .unwrap();
///     assert!(re.is_match(msg.as_ref()));
/// }
/// ```
pub trait Attach<Wrapper>: Sized {
    /// Records the caller's location in the error's backtrace.
    ///
    /// Uses `#[track_caller]` to automatically capture location.
    #[track_caller]
    #[inline]
    fn attach(self) -> Wrapper {
        self.attach_location(Location::caller())
    }

    /// Records an explicit location (rarely needed).
    fn attach_location(self, location: Location) -> Wrapper;
}

impl<E: Error2> Attach<Self> for E {
    #[inline]
    fn attach_location(mut self, location: Location) -> Self {
        self.backtrace_mut().push_location(location);
        self
    }
}

impl<T, E: Error2> Attach<Self> for Result<T, E> {
    #[inline]
    fn attach_location(self, location: Location) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.attach_location(location)),
        }
    }
}

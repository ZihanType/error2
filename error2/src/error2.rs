use std::{convert::Infallible, error::Error};

use crate::Backtrace;

/// Core trait extending `std::error::Error` with backtrace support.
///
/// `Error2` provides methods to access and modify the error's backtrace,
/// enabling detailed error propagation tracking across your application.
///
/// # Implementation
///
/// Typically implemented via `#[derive(Error2)]` macro:
///
/// ```
/// use error2::prelude::*;
///
/// #[derive(Debug, Error2)]
/// #[error2(display("configuration error"))]
/// struct ConfigError {
///     backtrace: Backtrace,
/// }
/// ```
///
/// The macro automatically generates the trait implementation and manages
/// the backtrace field.
///
/// # Accessing Backtrace
///
/// Use the `.backtrace()` method to access error chain and locations:
///
/// ```
/// # use error2::prelude::*;
/// # use std::io;
/// # #[derive(Debug, Error2)]
/// # #[error2(display("test"))]
/// # struct MyError { source: io::Error, backtrace: Backtrace }
/// # fn do_something() -> Result<(), MyError> { Ok(()) }
/// if let Err(e) = do_something() {
///     println!("{}", e.backtrace().error_message());
/// }
/// ```
///
/// See [`Backtrace`] for details on accessing error information.
pub trait Error2: Error {
    /// Returns a reference to the error's backtrace.
    ///
    /// The backtrace contains the complete error chain and location information.
    fn backtrace(&self) -> &Backtrace;

    /// Returns a mutable reference to the error's backtrace.
    ///
    /// Used internally by the library to record error propagation locations.
    fn backtrace_mut(&mut self) -> &mut Backtrace;
}

impl Error2 for Infallible {
    fn backtrace(&self) -> &Backtrace {
        match *self {}
    }

    fn backtrace_mut(&mut self) -> &mut Backtrace {
        match *self {}
    }
}

impl<T: Error2> Error2 for Box<T> {
    #[inline]
    fn backtrace(&self) -> &Backtrace {
        self.as_ref().backtrace()
    }

    #[inline]
    fn backtrace_mut(&mut self) -> &mut Backtrace {
        self.as_mut().backtrace_mut()
    }
}

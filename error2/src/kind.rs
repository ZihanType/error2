/// Represents the kind of error after downcasting [`BoxedError2`](crate::BoxedError2).
///
/// Used to distinguish between standard library errors and Error2 errors
/// when downcasting type-erased errors.
///
/// # Variants
///
/// - `Std`: Error from `std::error::Error` (has its own backtrace)
/// - `Err2`: Error implementing [`Error2`](crate::Error2) (reuses parent backtrace)
///
/// # Example
///
/// ```
/// use std::io;
///
/// use error2::{kind::ErrorKind, prelude::*};
///
/// # fn example() -> Result<(), BoxedError2> {
/// let err = std::io::Error::from(std::io::ErrorKind::NotFound);
/// let boxed = BoxedError2::from_std(err);
///
/// // Downcast to get the ErrorKind
/// match boxed.downcast_ref::<io::Error>() {
///     Some(ErrorKind::Std { source, backtrace }) => {
///         println!("Std error: {}", source);
///     }
///     Some(ErrorKind::Err2 { source }) => {
///         println!("Error2: {}", source);
///     }
///     None => {}
/// }
/// # Ok(())
/// # }
/// ```
pub enum ErrorKind<E, B> {
    /// Standard library error with its own backtrace.
    Std {
        /// The underlying error.
        source: E,
        /// The error's backtrace.
        backtrace: B,
    },
    /// Error2 error that reuses the parent's backtrace.
    Err2 {
        /// The underlying Error2 instance.
        source: E,
    },
}

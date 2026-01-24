use crate::{Error2, Location, transform::SourceToTarget};

/// Convenience methods for creating root errors.
///
/// `RootError` provides `.build()` and `.fail()` for creating errors that
/// represent new error origins (not wrapping other errors).
///
/// # Root Error Pattern
///
/// A root error only has a `backtrace` field (no `source`):
///
/// ```
/// use error2::prelude::*;
///
/// #[derive(Debug, Error2)]
/// pub enum AppError {
///     #[error2(display("invalid ID: {id}"))]
///     InvalidId { id: i64, backtrace: Backtrace },
///
///     #[error2(display("channel closed"))]
///     ChannelClosed { backtrace: Backtrace },
/// }
/// ```
///
/// # Using .build()
///
/// Create an error instance:
///
/// ```
/// # use error2::prelude::*;
/// # #[derive(Debug, Error2)]
/// # pub enum AppError {
/// #     #[error2(display("invalid ID: {id}"))]
/// #     InvalidId { id: i64, backtrace: Backtrace },
/// # }
/// # fn validate(id: i64) -> Result<(), AppError> {
/// if id < 0 {
///     let error = InvalidId2 { id }.build();
///     // Use error...
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Using .fail()
///
/// Create and return an error in one step:
///
/// ```
/// # use error2::prelude::*;
/// # #[derive(Debug, Error2)]
/// # pub enum AppError {
/// #     #[error2(display("invalid ID: {id}"))]
/// #     InvalidId { id: i64, backtrace: Backtrace },
/// # }
/// fn validate(id: i64) -> Result<(), AppError> {
///     if id < 0 {
///         return InvalidId2 { id }.fail();
///     }
///     Ok(())
/// }
/// ```
///
/// # Automatic Location Tracking
///
/// Both `.build()` and `.fail()` automatically capture the caller's
/// location using `#[track_caller]`, which is included in the backtrace.
///
/// ```
/// # use error2::prelude::*;
/// # #[derive(Debug, Error2)]
/// # pub enum AppError {
/// #     #[error2(display("invalid ID: {id}"))]
/// #     InvalidId { id: i64, backtrace: Backtrace },
/// # }
/// use regex::Regex;
///
/// let err = InvalidId2 { id: -1 }.build();
/// let msg = err.backtrace().error_message();
///
/// // Full error format with location tracking:
/// // AppError: invalid ID: -1
/// //     at /path/to/file.rs:683:33
///
/// let re = Regex::new(concat!(
///     r"(?s)^.+AppError: invalid ID: -1",
///     r"\n    at .+\.rs:\d+:\d+$",
/// ))
/// .unwrap();
/// assert!(re.is_match(msg.as_ref()));
/// ```
pub trait RootError<M, Target: Error2>: SourceToTarget<M, (), (), Target> + Sized {
    /// Creates a root error instance.
    ///
    /// Automatically captures the caller's location.
    #[inline]
    #[must_use]
    #[track_caller]
    fn build(self) -> Target {
        self.build_with_location(Location::caller())
    }

    /// Creates a root error with explicit location.
    #[inline]
    #[must_use]
    fn build_with_location(self, location: Location) -> Target {
        <Self as SourceToTarget<M, (), (), Target>>::source_to_target(self, (), location)
    }

    /// Creates and returns a root error as `Err(...)`.
    ///
    /// Convenient for returning errors directly.
    #[inline]
    #[track_caller]
    fn fail<T>(self) -> Result<T, Target> {
        Err(self.build())
    }

    /// Creates and returns error with explicit location.
    #[inline]
    fn fail_with_location<T>(self, location: Location) -> Result<T, Target> {
        Err(self.build_with_location(location))
    }
}

impl<M, Target, C> RootError<M, Target> for C
where
    Target: Error2,
    C: SourceToTarget<M, (), (), Target>,
{
}

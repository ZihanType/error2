mod root_err;
mod std_err;

use std::{
    any::TypeId,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use self::{root_err::RootErr, std_err::StdErr};
use crate::{Backtrace, Error2, Location, kind::ErrorKind, private, transform::SourceToTarget};

/// Type-erased error with automatic backtrace tracking.
///
/// `BoxedError2` provides anyhow-like ergonomics with type erasure,
/// allowing you to work with errors without specifying concrete types.
///
/// # Features
///
/// - **Type Erasure**: Store any error implementing `Error2` or `std::error::Error`
/// - **Downcasting**: Retrieve the original error type
/// - **Backtrace**: Access complete error chain and locations
///
/// # Conversion Control
///
/// Use `Via*` types to control how errors are wrapped:
///
/// ```
/// use error2::prelude::*;
///
/// fn example() -> Result<(), BoxedError2> {
///     // Wrap std::error::Error
///     std::fs::read_to_string("file.txt").context(ViaStd)?;
///
///     Ok(())
/// }
/// ```
///
/// See [`ViaRoot`], [`ViaStd`], and [`ViaErr2`] for details.
///
/// # Downcasting
///
/// Retrieve the original error type:
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
/// match boxed.downcast_ref::<io::Error>() {
///     Some(ErrorKind::Std { source, .. }) => {
///         println!("IO error: {}", source);
///     }
///     _ => {}
/// }
/// # Ok(())
/// # }
/// ```
pub struct BoxedError2 {
    source: Box<dyn Error2 + Send + Sync + 'static>,
}

impl BoxedError2 {
    #[inline]
    const fn source_ref(&self) -> &(dyn Error + Send + Sync + 'static) {
        &*self.source
    }

    #[inline]
    const fn source_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        &mut *self.source
    }

    #[inline]
    fn source(self) -> Box<dyn Error + Send + Sync + 'static> {
        self.source
    }

    /// Checks if this is a root error (not wrapping another error).
    #[inline]
    pub fn is_root(&self) -> bool {
        self.source_ref().is::<RootErr>()
    }

    fn generic_is_root<T: Error + 'static>() -> bool {
        TypeId::of::<RootErr>() == TypeId::of::<T>()
    }

    /// Checks if the boxed error contains an error of type `T`.
    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        debug_assert!(!Self::generic_is_root::<T>());
        let source = self.source_ref();

        source.is::<StdErr<T>>() || source.is::<T>()
    }

    /// Attempts to downcast to a reference of type `T`.
    ///
    /// Returns `Some(ErrorKind)` if the error is of type `T`.
    #[inline]
    pub fn downcast_ref<T: Error + 'static>(&self) -> Option<ErrorKind<&T, &Backtrace>> {
        debug_assert!(!Self::generic_is_root::<T>());
        let source = self.source_ref();

        if let Some(StdErr { source, backtrace }) = source.downcast_ref::<StdErr<T>>() {
            Some(ErrorKind::Std { source, backtrace })
        } else if let Some(source) = source.downcast_ref::<T>() {
            Some(ErrorKind::Err2 { source })
        } else {
            None
        }
    }

    /// Attempts to downcast to a mutable reference of type `T`.
    #[inline]
    pub fn downcast_mut<T: Error + 'static>(
        &mut self,
    ) -> Option<ErrorKind<&mut T, &mut Backtrace>> {
        debug_assert!(!Self::generic_is_root::<T>());
        let source = self.source_ref();

        if source.is::<StdErr<T>>() {
            let StdErr { source, backtrace } =
                self.source_mut().downcast_mut::<StdErr<T>>().unwrap();
            Some(ErrorKind::Std { source, backtrace })
        } else if source.is::<T>() {
            let source = self.source_mut().downcast_mut::<T>().unwrap();
            Some(ErrorKind::Err2 { source })
        } else {
            None
        }
    }

    /// Attempts to downcast to an owned value of type `T`.
    ///
    /// Returns `Ok(ErrorKind)` if successful, or `Err(self)` on failure.
    #[inline]
    pub fn downcast<T: Error + 'static>(self) -> Result<ErrorKind<T, Backtrace>, Self> {
        debug_assert!(!Self::generic_is_root::<T>());
        let source = self.source_ref();

        if source.is::<StdErr<T>>() {
            let StdErr { source, backtrace } = *self.source().downcast::<StdErr<T>>().unwrap();
            Ok(ErrorKind::Std { source, backtrace })
        } else if source.is::<T>() {
            let source = *self.source().downcast::<T>().unwrap();
            Ok(ErrorKind::Err2 { source })
        } else {
            Err(self)
        }
    }
}

impl Display for BoxedError2 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.source, f)
    }
}

impl Debug for BoxedError2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            Debug::fmt(&self.source, f)
        } else {
            Display::fmt(&self.source, f)?;
            write!(f, "\n\n")?;

            let m = self.backtrace().error_message();
            Display::fmt(&m, f)
        }
    }
}

impl Error for BoxedError2 {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if self.is_root() {
            None
        } else {
            Some(&*self.source)
        }
    }
}

impl Error2 for BoxedError2 {
    #[inline]
    fn backtrace(&self) -> &Backtrace {
        self.source.backtrace()
    }

    #[inline]
    fn backtrace_mut(&mut self) -> &mut Backtrace {
        self.source.backtrace_mut()
    }
}

impl BoxedError2 {
    /// Creates a `BoxedError2` from a root error message.
    ///
    /// Use for errors that don't wrap other errors.
    #[track_caller]
    #[inline]
    pub fn from_root<R>(root: R) -> BoxedError2
    where
        R: Display + Debug + Send + Sync + 'static,
    {
        Self::from_root_with_location(root, Location::caller())
    }

    /// Creates a root error with explicit location.
    pub fn from_root_with_location<R>(root: R, location: Location) -> BoxedError2
    where
        R: Display + Debug + Send + Sync + 'static,
    {
        let mut error = BoxedError2 {
            source: Box::new(RootErr::new(root)),
        };

        crate::push_error(&mut error, location);

        error
    }

    /// Creates a `BoxedError2` from a `std::error::Error`.
    ///
    /// Wraps standard library or third-party errors.
    #[track_caller]
    #[inline]
    pub fn from_std<T>(source: T) -> BoxedError2
    where
        T: Error + Send + Sync + 'static,
    {
        Self::from_std_with_location(source, Location::caller())
    }

    /// Creates from std::error::Error with explicit location.
    pub fn from_std_with_location<T>(source: T, location: Location) -> BoxedError2
    where
        T: Error + Send + Sync + 'static,
    {
        if (&source as &(dyn Error + Send + Sync)).is::<BoxedError2>() {
            let mut e =
                <dyn Error + Send + Sync>::downcast::<BoxedError2>(Box::new(source)).unwrap();

            e.backtrace_mut().push_location(location);
            *e
        } else {
            let mut error = BoxedError2 {
                source: Box::new(StdErr::new(source)),
            };

            crate::push_error(&mut error, location);

            error
        }
    }

    /// Creates a `BoxedError2` from an `Error2` type.
    ///
    /// Preserves the original error's backtrace.
    #[track_caller]
    #[inline]
    pub fn from_err2<T>(source: T) -> BoxedError2
    where
        T: Error2 + Send + Sync + 'static,
    {
        Self::from_err2_with_location(source, Location::caller())
    }

    /// Creates from Error2 with explicit location.
    pub fn from_err2_with_location<T>(source: T, location: Location) -> BoxedError2
    where
        T: Error2 + Send + Sync + 'static,
    {
        if (&source as &(dyn Error + Send + Sync)).is::<BoxedError2>() {
            let mut e =
                <dyn Error + Send + Sync>::downcast::<BoxedError2>(Box::new(source)).unwrap();

            e.backtrace_mut().push_location(location);
            *e
        } else {
            let mut error = BoxedError2 {
                source: Box::new(source),
            };

            crate::push_error(&mut error, location);

            error
        }
    }
}

/// Wrapper for creating root errors with [`BoxedError2`].
///
/// Use with `.context()` to create errors from non-Error types
/// like strings or custom messages.
///
/// # Example
///
/// ```
/// use error2::prelude::*;
///
/// fn validate(value: i32) -> Result<(), BoxedError2> {
///     if value < 0 {
///         // Create a root error from a string message
///         ViaRoot("validation failed: negative value").fail()?;
///     }
///     Ok(())
/// }
///
/// # fn main() -> Result<(), BoxedError2> {
/// # assert!(validate(10).is_ok());
/// # assert!(validate(-5).is_err());
/// # Ok(())
/// # }
/// ```
pub struct ViaRoot<M>(pub M)
where
    M: Display + Debug + Send + Sync + 'static;

impl<M> SourceToTarget<private::ViaFull, (), (), BoxedError2> for ViaRoot<M>
where
    M: Display + Debug + Send + Sync + 'static,
{
    #[inline]
    fn source_to_target(self, _source: (), location: Location) -> BoxedError2 {
        BoxedError2::from_root_with_location(self.0, location)
    }
}

/// Wrapper for converting `std::error::Error` to [`BoxedError2`].
///
/// Use with `.context()` to wrap standard library or third-party errors.
///
/// # Example
///
/// ```
/// use std::fs;
///
/// use error2::prelude::*;
///
/// fn read_file() -> Result<String, BoxedError2> {
///     fs::read_to_string("file.txt").context(ViaStd)
/// }
/// ```
pub struct ViaStd;

impl<T> SourceToTarget<private::ViaFull, T, T, BoxedError2> for ViaStd
where
    T: Error + Send + Sync + 'static,
{
    #[inline]
    fn source_to_target(self, source: T, location: Location) -> BoxedError2 {
        BoxedError2::from_std_with_location(source, location)
    }
}

/// Wrapper for converting `Error2` types to [`BoxedError2`].
///
/// Use with `.context()` to wrap Error2-based errors, preserving
/// the original backtrace.
///
/// # Example
///
/// ```
/// use error2::prelude::*;
///
/// # #[derive(Debug, Error2)]
/// # #[error2(display("inner error"))]
/// # struct InnerError { backtrace: Backtrace }
/// # fn inner() -> Result<(), InnerError> { Ok(()) }
/// fn outer() -> Result<(), BoxedError2> {
///     inner().context(ViaErr2)?;
///     Ok(())
/// }
/// ```
pub struct ViaErr2;

impl<T> SourceToTarget<private::ViaFull, T, T, BoxedError2> for ViaErr2
where
    T: Error2 + Send + Sync + 'static,
{
    #[inline]
    fn source_to_target(self, source: T, location: Location) -> BoxedError2 {
        BoxedError2::from_err2_with_location(source, location)
    }
}

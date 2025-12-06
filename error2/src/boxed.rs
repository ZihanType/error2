use std::{error::Error, fmt};

use crate::{Backtrace, Error2, ErrorWrap, Location, NoneError};

struct StringError(Box<str>);

impl fmt::Display for StringError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for StringError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Error for StringError {}

pub struct BoxedError2 {
    source: Box<dyn Error + Send + Sync + 'static>,
    backtrace: Backtrace,
}

impl fmt::Display for BoxedError2 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, f)
    }
}

impl fmt::Debug for BoxedError2 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.source, f)
    }
}

impl Error for BoxedError2 {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if self.source.is::<StringError>() {
            None
        } else {
            Some(&*self.source)
        }
    }
}

impl Error2 for BoxedError2 {
    #[inline]
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    #[inline]
    fn backtrace_mut(&mut self) -> &mut Backtrace {
        &mut self.backtrace
    }
}

impl BoxedError2 {
    #[track_caller]
    #[inline]
    pub fn from_msg<S>(s: S) -> BoxedError2
    where
        S: Into<String>,
    {
        Self::from_msg_with_location(s, Location::caller())
    }

    pub fn from_msg_with_location<S>(s: S, location: Location) -> BoxedError2
    where
        S: Into<String>,
    {
        fn inner(s: String, location: Location) -> BoxedError2 {
            let mut error = BoxedError2 {
                source: Box::new(StringError(s.into())),
                backtrace: Backtrace::new(),
            };

            error.push_error(location);

            error
        }

        let s: String = s.into();
        inner(s, location)
    }

    #[track_caller]
    #[inline]
    pub fn from_std<T>(source: T) -> BoxedError2
    where
        T: Error + Send + Sync + 'static,
    {
        Self::from_std_with_location(source, Location::caller())
    }

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
            let backtrace = Backtrace::with_head(&source);

            let mut error = BoxedError2 {
                source: Box::new(source),
                backtrace,
            };

            error.push_error(location);

            error
        }
    }

    #[track_caller]
    #[inline]
    pub fn from_err2<T>(source: T) -> BoxedError2
    where
        T: Error2 + Send + Sync + 'static,
    {
        Self::from_err2_with_location(source, Location::caller())
    }

    pub fn from_err2_with_location<T>(mut source: T, location: Location) -> BoxedError2
    where
        T: Error2 + Send + Sync + 'static,
    {
        if (&source as &(dyn Error + Send + Sync)).is::<BoxedError2>() {
            let mut e =
                <dyn Error + Send + Sync>::downcast::<BoxedError2>(Box::new(source)).unwrap();

            e.backtrace_mut().push_location(location);
            *e
        } else {
            let backtrace = source.backtrace_mut().take();

            let mut error = BoxedError2 {
                source: Box::new(source),
                backtrace,
            };

            error.push_error(location);

            error
        }
    }
}

pub struct ViaNone<S: Into<String>>(pub S);

impl<S: Into<String>> ViaNone<S> {
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn build(self) -> BoxedError2 {
        self.build_with_location(Location::caller())
    }

    #[inline]
    #[must_use]
    pub fn build_with_location(self, location: Location) -> BoxedError2 {
        <Self as ErrorWrap<NoneError, BoxedError2>>::wrap(self, NoneError, location)
    }

    #[inline]
    #[track_caller]
    pub fn fail<T>(self) -> Result<T, BoxedError2> {
        self.fail_with_location(Location::caller())
    }

    #[inline]
    pub fn fail_with_location<T>(self, location: Location) -> Result<T, BoxedError2> {
        Err(self.build_with_location(location))
    }
}

impl<S> ErrorWrap<NoneError, BoxedError2> for ViaNone<S>
where
    S: Into<String>,
{
    #[inline]
    fn wrap(self, _source: NoneError, location: Location) -> BoxedError2 {
        BoxedError2::from_msg_with_location(self.0, location)
    }
}

pub struct ViaStd;

impl<T> ErrorWrap<T, BoxedError2> for ViaStd
where
    T: Error + Send + Sync + 'static,
{
    #[inline]
    fn wrap(self, source: T, location: Location) -> BoxedError2 {
        BoxedError2::from_std_with_location(source, location)
    }
}

pub struct ViaErr2;

impl<T> ErrorWrap<T, BoxedError2> for ViaErr2
where
    T: Error2 + Send + Sync + 'static,
{
    #[inline]
    fn wrap(self, source: T, location: Location) -> BoxedError2 {
        BoxedError2::from_err2_with_location(source, location)
    }
}

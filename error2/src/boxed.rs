use std::{error::Error, fmt};

use crate::{Backtrace, Error2, ErrorFullWrap, Location, NoneError, private};

struct StringError {
    s: Box<str>,
    backtrace: Backtrace,
}

impl fmt::Display for StringError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.s, f)
    }
}

impl fmt::Debug for StringError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.s, f)
    }
}

impl Error for StringError {}

impl Error2 for StringError {
    #[inline]
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    #[inline]
    fn backtrace_mut(&mut self) -> &mut Backtrace {
        &mut self.backtrace
    }
}

struct StdErr<T> {
    source: T,
    backtrace: Backtrace,
}

impl<T: fmt::Display> fmt::Display for StdErr<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for StdErr<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.source, f)
    }
}

impl<T: Error> Error for StdErr<T> {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Error::source(&self.source)
    }
}

impl<T: Error> Error2 for StdErr<T> {
    #[inline]
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    #[inline]
    fn backtrace_mut(&mut self) -> &mut Backtrace {
        &mut self.backtrace
    }
}

pub struct BoxedError2 {
    source: Box<dyn Error2 + Send + Sync + 'static>,
}

impl BoxedError2 {
    #[inline]
    const fn as_err(&self) -> &(dyn Error + Send + Sync + 'static) {
        &*self.source
    }

    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        let err = self.as_err();

        if err.is::<StdErr<T>>() {
            true
        } else {
            err.is::<T>()
        }
    }
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
        if self.is::<StringError>() {
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
                source: Box::new(StringError {
                    s: s.into(),
                    backtrace: Backtrace::new(),
                }),
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
                source: Box::new(StdErr { source, backtrace }),
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

            error.push_error(location);

            error
        }
    }
}

pub struct ViaNone<S: Into<String>>(pub S);

impl<S> ErrorFullWrap<private::ViaFull, NoneError, NoneError, BoxedError2> for ViaNone<S>
where
    S: Into<String>,
{
    #[inline]
    fn full_wrap(self, _source: NoneError, location: Location) -> BoxedError2 {
        BoxedError2::from_msg_with_location(self.0, location)
    }
}

pub struct ViaStd;

impl<T> ErrorFullWrap<private::ViaFull, T, T, BoxedError2> for ViaStd
where
    T: Error + Send + Sync + 'static,
{
    #[inline]
    fn full_wrap(self, source: T, location: Location) -> BoxedError2 {
        BoxedError2::from_std_with_location(source, location)
    }
}

pub struct ViaErr2;

impl<T> ErrorFullWrap<private::ViaFull, T, T, BoxedError2> for ViaErr2
where
    T: Error2 + Send + Sync + 'static,
{
    #[inline]
    fn full_wrap(self, source: T, location: Location) -> BoxedError2 {
        BoxedError2::from_err2_with_location(source, location)
    }
}

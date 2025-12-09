mod root_err;
mod std_err;

use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use self::{root_err::RootErr, std_err::StdErr};
use crate::{Backtrace, Error2, Location, NoneError, SourceToTarget, private};

pub struct BoxedError2 {
    source: Box<dyn Error2 + Send + Sync + 'static>,
}

impl BoxedError2 {
    #[inline]
    const fn source(&self) -> &(dyn Error + Send + Sync + 'static) {
        &*self.source
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.source().is::<RootErr>()
    }

    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        let source = self.source();

        source.is::<StdErr<T>>() || source.is::<T>()
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

            let m = crate::extract_error_message(self);
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
    #[track_caller]
    #[inline]
    pub fn from_root<R>(root: R) -> BoxedError2
    where
        R: Display + Debug + Send + Sync + 'static,
    {
        Self::from_root_with_location(root, Location::caller())
    }

    pub fn from_root_with_location<R>(root: R, location: Location) -> BoxedError2
    where
        R: Display + Debug + Send + Sync + 'static,
    {
        let mut error = BoxedError2 {
            source: Box::new(RootErr::new(root)),
        };

        error.push_error(location);

        error
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
            let mut error = BoxedError2 {
                source: Box::new(StdErr::new(source)),
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

pub struct ViaRoot<M>(pub M)
where
    M: Display + Debug + Send + Sync + 'static;

impl<M> SourceToTarget<private::ViaFull, NoneError, NoneError, BoxedError2> for ViaRoot<M>
where
    M: Display + Debug + Send + Sync + 'static,
{
    #[inline]
    fn source_to_target(self, _source: NoneError, location: Location) -> BoxedError2 {
        BoxedError2::from_root_with_location(self.0, location)
    }
}

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

mod root_err;
mod std_err;

use std::{
    any::TypeId,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use self::{root_err::RootErr, std_err::StdErr};
use crate::{Backtrace, Error2, Location, private, transform::SourceToTarget};

pub enum ErrorKind<E, B> {
    Std { source: E, backtrace: B },
    Err2 { source: E },
}

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

    #[inline]
    pub fn is_root(&self) -> bool {
        self.source_ref().is::<RootErr>()
    }

    fn generic_is_root<T: Error + 'static>() -> bool {
        TypeId::of::<RootErr>() == TypeId::of::<T>()
    }

    #[inline]
    pub fn is<T: Error + 'static>(&self) -> bool {
        debug_assert!(!Self::generic_is_root::<T>());
        let source = self.source_ref();

        source.is::<StdErr<T>>() || source.is::<T>()
    }

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

        crate::push_error(&mut error, location);

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

            crate::push_error(&mut error, location);

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

            crate::push_error(&mut error, location);

            error
        }
    }
}

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

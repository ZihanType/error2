use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use crate::{Backtrace, Error2};

pub(super) struct StdErr<T> {
    pub(super) source: T,
    pub(super) backtrace: Backtrace,
}

impl<T> StdErr<T> {
    pub(super) fn new(source: T) -> Self
    where
        T: Error + Send + Sync + 'static,
    {
        let backtrace = Backtrace::with_head(&source);
        Self { source, backtrace }
    }
}

impl<T: Display> Display for StdErr<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.source, f)
    }
}

impl<T: Debug> Debug for StdErr<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.source, f)
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

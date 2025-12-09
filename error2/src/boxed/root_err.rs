use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use crate::{Backtrace, Error2};

trait Root: Display + Debug + Send + Sync {}

impl<T: Display + Debug + Send + Sync> Root for T {}

pub(super) struct RootErr {
    r: Box<dyn Root>,
    backtrace: Backtrace,
}

impl RootErr {
    pub(super) fn new<R>(root: R) -> Self
    where
        R: Display + Debug + Send + Sync + 'static,
    {
        Self {
            r: Box::new(root),
            backtrace: Backtrace::new(),
        }
    }
}

impl Display for RootErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&*self.r, f)
    }
}

impl Debug for RootErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&*self.r, f)
    }
}

impl Error for RootErr {}

impl Error2 for RootErr {
    #[inline]
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    #[inline]
    fn backtrace_mut(&mut self) -> &mut Backtrace {
        &mut self.backtrace
    }
}

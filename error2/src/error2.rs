use std::{convert::Infallible, error::Error};

use crate::Backtrace;

pub trait Error2: Error {
    fn backtrace(&self) -> &Backtrace;

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

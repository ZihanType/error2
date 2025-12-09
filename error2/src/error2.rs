use std::{any, convert::Infallible, error::Error};

use crate::{Backtrace, Location};

pub trait Error2: Error {
    fn backtrace(&self) -> &Backtrace;

    fn backtrace_mut(&mut self) -> &mut Backtrace;

    fn error_message(&self) -> Box<str> {
        crate::extract_error_message(self.backtrace())
    }

    #[doc(hidden)]
    fn push_error(&mut self, location: Location) {
        let type_name = any::type_name::<Self>();
        let display = self.to_string();

        self.backtrace_mut()
            .push_error(type_name, display, location);
    }
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

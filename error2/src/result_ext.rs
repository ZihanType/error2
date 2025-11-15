use std::error::Error;

use crate::{Error2, ErrorWrap, Location};

pub trait ResultExt<T, E1>: Sized {
    #[track_caller]
    fn context<C, E2, E3>(self, context: C) -> Result<T, E3>
    where
        E1: Into<E2>,
        C: ErrorWrap<E2, E3>,
        E2: Error,
        E3: Error2,
    {
        self.context_and_location(context, Location::caller())
    }

    fn context_and_location<C, E2, E3>(self, context: C, location: Location) -> Result<T, E3>
    where
        E1: Into<E2>,
        C: ErrorWrap<E2, E3>,
        E2: Error,
        E3: Error2;

    #[track_caller]
    fn with_context<F, C, E2, E3>(self, f: F) -> Result<T, E3>
    where
        F: FnOnce() -> C,
        E1: Into<E2>,
        C: ErrorWrap<E2, E3>,
        E2: Error,
        E3: Error2,
    {
        self.with_context_and_location(f, Location::caller())
    }

    fn with_context_and_location<F, C, E2, E3>(self, f: F, location: Location) -> Result<T, E3>
    where
        F: FnOnce() -> C,
        E1: Into<E2>,
        C: ErrorWrap<E2, E3>,
        E2: Error,
        E3: Error2;
}

impl<T, E1> ResultExt<T, E1> for Result<T, E1> {
    fn context_and_location<C, E2, E3>(self, context: C, location: Location) -> Result<T, E3>
    where
        E1: Into<E2>,
        C: ErrorWrap<E2, E3>,
        E2: Error,
        E3: Error2,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e1) => {
                let e2: E2 = e1.into();
                Err(context.wrap(e2, location))
            }
        }
    }

    fn with_context_and_location<F, C, E2, E3>(self, f: F, location: Location) -> Result<T, E3>
    where
        F: FnOnce() -> C,
        E1: Into<E2>,
        C: ErrorWrap<E2, E3>,
        E2: Error,
        E3: Error2,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e1) => {
                let e2: E2 = e1.into();
                let context = f();
                Err(context.wrap(e2, location))
            }
        }
    }
}

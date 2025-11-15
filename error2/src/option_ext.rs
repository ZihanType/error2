use std::{error::Error, fmt};

use crate::{Error2, ErrorWrap, Location};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoneError;

impl fmt::Display for NoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NoneError")
    }
}

impl Error for NoneError {}

pub trait OptionExt<T>: Sized {
    #[track_caller]
    fn context<C, E>(self, context: C) -> Result<T, E>
    where
        C: ErrorWrap<NoneError, E>,
        E: Error2,
    {
        self.context_and_location(context, Location::caller())
    }

    fn context_and_location<C, E>(self, context: C, location: Location) -> Result<T, E>
    where
        C: ErrorWrap<NoneError, E>,
        E: Error2;

    #[track_caller]
    fn with_context<F, C, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: ErrorWrap<NoneError, E>,
        E: Error2,
    {
        self.with_context_and_location(f, Location::caller())
    }

    fn with_context_and_location<F, C, E>(self, f: F, location: Location) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: ErrorWrap<NoneError, E>,
        E: Error2;
}

impl<T> OptionExt<T> for Option<T> {
    fn context_and_location<C, E>(self, context: C, location: Location) -> Result<T, E>
    where
        C: ErrorWrap<NoneError, E>,
        E: Error2,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(context.wrap(NoneError, location)),
        }
    }

    fn with_context_and_location<F, C, E>(self, f: F, location: Location) -> Result<T, E>
    where
        F: FnOnce() -> C,
        C: ErrorWrap<NoneError, E>,
        E: Error2,
    {
        match self {
            Some(t) => Ok(t),
            None => {
                let context = f();
                Err(context.wrap(NoneError, location))
            }
        }
    }
}

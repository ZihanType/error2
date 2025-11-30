use std::error::Error;

use crate::{Error2, ErrorWrap, Location, NoneError};

pub trait Context<T, Source, Middle, Target, C>: Sized
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: ErrorWrap<Middle, Target>,
{
    #[track_caller]
    fn context(self, context: C) -> Result<T, Target> {
        self.context_and_location(context, Location::caller())
    }

    fn context_and_location(self, context: C, location: Location) -> Result<T, Target>;

    #[track_caller]
    fn with_context<F>(self, f: F) -> Result<T, Target>
    where
        F: FnOnce() -> C,
    {
        self.with_context_and_location(f, Location::caller())
    }

    fn with_context_and_location<F>(self, f: F, location: Location) -> Result<T, Target>
    where
        F: FnOnce() -> C;
}

impl<T, Source, Middle, Target, C> Context<T, Source, Middle, Target, C> for Source
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: ErrorWrap<Middle, Target>,
{
    fn context_and_location(self, context: C, location: Location) -> Result<T, Target> {
        let middle: Middle = self.into();
        Err(context.wrap(middle, location))
    }

    fn with_context_and_location<F>(self, f: F, location: Location) -> Result<T, Target>
    where
        F: FnOnce() -> C,
    {
        let context = f();
        let middle: Middle = self.into();
        Err(context.wrap(middle, location))
    }
}

impl<T, Target, C> Context<T, NoneError, NoneError, Target, C> for Option<T>
where
    Target: Error2,
    C: ErrorWrap<NoneError, Target>,
{
    #[inline]
    fn context_and_location(self, context: C, location: Location) -> Result<T, Target> {
        match self {
            Some(t) => Ok(t),
            None => NoneError.context_and_location(context, location),
        }
    }

    #[inline]
    fn with_context_and_location<F>(self, f: F, location: Location) -> Result<T, Target>
    where
        F: FnOnce() -> C,
    {
        match self {
            Some(t) => Ok(t),
            None => NoneError.with_context_and_location(f, location),
        }
    }
}

impl<T, Source, Middle, Target, C> Context<T, Source, Middle, Target, C> for Result<T, Source>
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: ErrorWrap<Middle, Target>,
{
    #[inline]
    fn context_and_location(self, context: C, location: Location) -> Result<T, Target> {
        match self {
            Ok(t) => Ok(t),
            Err(source) => source.context_and_location(context, location),
        }
    }

    #[inline]
    fn with_context_and_location<F>(self, f: F, location: Location) -> Result<T, Target>
    where
        F: FnOnce() -> C,
    {
        match self {
            Ok(t) => Ok(t),
            Err(source) => source.with_context_and_location(f, location),
        }
    }
}

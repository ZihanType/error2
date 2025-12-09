use std::error::Error;

use crate::{Error2, Location, NoneError, SourceToTarget};

pub trait Context<T, M, Source, Middle, Target, C>: Sized
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: SourceToTarget<M, Source, Middle, Target>,
{
    #[inline]
    #[track_caller]
    fn context(self, context: C) -> Result<T, Target> {
        self.context_and_location(context, Location::caller())
    }

    fn context_and_location(self, context: C, location: Location) -> Result<T, Target>;

    #[inline]
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

impl<T, M, Source, Middle, Target, C> Context<T, M, Source, Middle, Target, C> for Source
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: SourceToTarget<M, Source, Middle, Target>,
{
    #[inline]
    fn context_and_location(self, context: C, location: Location) -> Result<T, Target> {
        Err(context.source_to_target(self, location))
    }

    #[inline]
    fn with_context_and_location<F>(self, f: F, location: Location) -> Result<T, Target>
    where
        F: FnOnce() -> C,
    {
        let context = f();
        Err(context.source_to_target(self, location))
    }
}

impl<T, M, Target, C> Context<T, M, NoneError, NoneError, Target, C> for Option<T>
where
    Target: Error2,
    C: SourceToTarget<M, NoneError, NoneError, Target>,
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

impl<T, M, Source, Middle, Target, C> Context<T, M, Source, Middle, Target, C> for Result<T, Source>
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: SourceToTarget<M, Source, Middle, Target>,
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

use std::error::Error;

use crate::{Error2, Location, private};

pub trait MiddleToTarget<Middle, Target>
where
    Middle: Error, // at least implement `Error` trait
    Target: Error2,
{
    fn middle_to_target(self, middle: Middle, location: Location) -> Target;
}

pub trait SourceToTarget<M, Source, Middle, Target>
where
    Source: Error + Into<Middle>, // at least implement `Error` trait
    Middle: Error,                // at least implement `Error` trait
    Target: Error2,
{
    fn source_to_target(self, source: Source, location: Location) -> Target;
}

impl<Source, Middle, Target, C> SourceToTarget<private::ViaPartial, Source, Middle, Target> for C
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: MiddleToTarget<Middle, Target>,
{
    #[inline]
    fn source_to_target(self, source: Source, location: Location) -> Target {
        let middle: Middle = source.into();
        self.middle_to_target(middle, location)
    }
}

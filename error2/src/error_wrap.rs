use std::error::Error;

use crate::{Error2, Location, private};

pub trait ErrorHalfWrap<Middle, Target>
where
    Middle: Error, // at least implement `Error` trait
    Target: Error2,
{
    fn half_wrap(self, middle: Middle, location: Location) -> Target;
}

pub trait ErrorFullWrap<M, Source, Middle, Target>
where
    Source: Error + Into<Middle>, // at least implement `Error` trait
    Middle: Error,                // at least implement `Error` trait
    Target: Error2,
{
    fn full_wrap(self, source: Source, location: Location) -> Target;
}

impl<Source, Middle, Target, C> ErrorFullWrap<private::ViaHalf, Source, Middle, Target> for C
where
    Source: Error + Into<Middle>,
    Middle: Error,
    Target: Error2,
    C: ErrorHalfWrap<Middle, Target>,
{
    #[inline]
    fn full_wrap(self, source: Source, location: Location) -> Target {
        let middle: Middle = source.into();
        self.half_wrap(middle, location)
    }
}

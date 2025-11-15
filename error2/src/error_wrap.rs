use std::error::Error;

use crate::{Error2, Location};

pub trait ErrorWrap<Source, Target>
where
    Source: Error, // at least implement `Error` trait
    Target: Error2,
{
    fn wrap(self, source: Source, location: Location) -> Target;
}

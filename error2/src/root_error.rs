use crate::{Error2, ErrorWrap, Location, NoneError};

pub trait RootError<Target: Error2>: ErrorWrap<NoneError, Target> + Sized {
    #[inline]
    #[must_use]
    #[track_caller]
    fn build(self) -> Target {
        self.build_with_location(Location::caller())
    }

    #[inline]
    #[must_use]
    fn build_with_location(self, location: Location) -> Target {
        <Self as ErrorWrap<NoneError, Target>>::wrap(self, NoneError, location)
    }

    #[inline]
    #[track_caller]
    fn fail<T>(self) -> Result<T, Target> {
        Err(self.build())
    }

    #[inline]
    fn fail_with_location<T>(self, location: Location) -> Result<T, Target> {
        Err(self.build_with_location(location))
    }
}

impl<Target, C> RootError<Target> for C
where
    Target: Error2,
    C: ErrorWrap<NoneError, Target>,
{
}

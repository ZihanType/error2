use crate::{Error2, Location, NoneError, SourceToTarget};

pub trait RootError<M, Target: Error2>:
    SourceToTarget<M, NoneError, NoneError, Target> + Sized
{
    #[inline]
    #[must_use]
    #[track_caller]
    fn build(self) -> Target {
        self.build_with_location(Location::caller())
    }

    #[inline]
    #[must_use]
    fn build_with_location(self, location: Location) -> Target {
        <Self as SourceToTarget<M, NoneError, NoneError, Target>>::source_to_target(
            self, NoneError, location,
        )
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

impl<M, Target, C> RootError<M, Target> for C
where
    Target: Error2,
    C: SourceToTarget<M, NoneError, NoneError, Target>,
{
}

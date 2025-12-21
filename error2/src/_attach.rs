use crate::{Error2, Location};

pub trait Attach<Wrapper>: Sized {
    #[track_caller]
    #[inline]
    fn attach(self) -> Wrapper {
        self.attach_location(Location::caller())
    }

    fn attach_location(self, location: Location) -> Wrapper;
}

impl<E: Error2> Attach<Self> for E {
    #[inline]
    fn attach_location(mut self, location: Location) -> Self {
        self.backtrace_mut().push_location(location);
        self
    }
}

impl<T, E: Error2> Attach<Self> for Result<T, E> {
    #[inline]
    fn attach_location(self, location: Location) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.attach_location(location)),
        }
    }
}

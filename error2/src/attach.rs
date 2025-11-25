use crate::{Error2, Location};

pub trait Attach: Sized {
    #[track_caller]
    #[inline]
    fn attach(self) -> Self {
        self.attach_location(Location::caller())
    }

    fn attach_location(self, location: Location) -> Self;
}

impl<E: Error2> Attach for E {
    #[inline]
    fn attach_location(mut self, location: Location) -> Self {
        self.backtrace_mut().attach_location(location);
        self
    }
}

impl<T, E: Error2> Attach for Result<T, E> {
    #[inline]
    fn attach_location(self, location: Location) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.attach_location(location)),
        }
    }
}

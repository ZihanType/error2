use crate::{Error2, Location};

pub trait Attach {
    fn attach(self) -> Self;
    fn attach_location(self, location: Location) -> Self;
}

impl<E: Error2> Attach for E {
    #[track_caller]
    #[inline]
    fn attach(self) -> Self {
        self.attach_location(Location::caller())
    }

    #[inline]
    fn attach_location(mut self, location: Location) -> Self {
        self.backtrace_mut().attach_location(location);
        self
    }
}

impl<T, E: Error2> Attach for Result<T, E> {
    #[track_caller]
    #[inline]
    fn attach(self) -> Self {
        self.attach_location(Location::caller())
    }

    #[inline]
    fn attach_location(self, location: Location) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(mut e) => {
                e.backtrace_mut().attach_location(location);
                Err(e)
            }
        }
    }
}

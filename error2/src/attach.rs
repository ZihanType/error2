use crate::{Error2, Location};

pub trait Attach {
    #[track_caller]
    fn attach(self) -> Self;
    fn attach_location(self, location: Location) -> Self;
}

impl<E: Error2> Attach for E {
    #[track_caller]
    #[inline]
    fn attach(mut self) -> Self {
        self.locations().attach();
        self
    }

    #[inline]
    fn attach_location(mut self, location: Location) -> Self {
        self.locations().attach_location(location);
        self
    }
}

impl<T, E: Error2> Attach for Result<T, E> {
    #[track_caller]
    #[inline]
    fn attach(self) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(mut e) => {
                e.locations().attach();
                Err(e)
            }
        }
    }

    #[inline]
    fn attach_location(self, location: Location) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(mut e) => {
                e.locations().attach_location(location);
                Err(e)
            }
        }
    }
}

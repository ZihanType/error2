use crate::{ErrorExt, Location};

pub trait Attach {
    #[track_caller]
    fn attach(self) -> Self;
    fn attach_location(self, location: Location) -> Self;
}

impl<E: ErrorExt> Attach for E {
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

impl<T, E: ErrorExt> Attach for Result<T, E> {
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

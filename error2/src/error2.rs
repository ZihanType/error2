use std::{convert::Infallible, error::Error};

use crate::{Locations, NextError};

pub trait Error2: Error {
    fn entry(&self) -> (&Locations, NextError<'_>);

    fn locations(&mut self) -> &mut Locations;
}

impl Error2 for Infallible {
    fn entry(&self) -> (&Locations, NextError<'_>) {
        match *self {}
    }

    fn locations(&mut self) -> &mut Locations {
        match *self {}
    }
}

impl<T: Error2> Error2 for Box<T> {
    #[inline]
    fn entry(&self) -> (&Locations, NextError<'_>) {
        self.as_ref().entry()
    }

    #[inline]
    fn locations(&mut self) -> &mut Locations {
        self.as_mut().locations()
    }
}

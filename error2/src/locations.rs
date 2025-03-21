use std::fmt;

use crate::Location;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locations(Vec<Location>);

impl Locations {
    #[doc(hidden)]
    #[inline]
    pub fn new(location: Location) -> Self {
        Self(vec![location])
    }

    #[track_caller]
    #[inline]
    pub fn caller() -> Self {
        Self(vec![Location::caller()])
    }

    #[inline]
    pub fn inner(&self) -> &[Location] {
        &self.0
    }

    #[inline]
    pub fn into_inner(self) -> Vec<Location> {
        self.0
    }

    #[track_caller]
    #[inline]
    pub fn attach(&mut self) {
        self.0.push(Location::caller());
    }

    #[inline]
    pub fn attach_location(&mut self, location: Location) {
        self.0.push(location);
    }
}

impl fmt::Debug for Locations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl From<Locations> for Vec<Location> {
    fn from(locations: Locations) -> Self {
        locations.into_inner()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "snafu")))]
#[cfg(feature = "snafu")]
impl snafu::GenerateImplicitData for Locations {
    #[track_caller]
    #[inline]
    fn generate() -> Self {
        Self::caller()
    }
}

use crate::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct DoubleLocations([Location; 2]);

impl DoubleLocations {
    pub(super) const fn new(location: Location) -> Self {
        Self([location, Location::uninit()])
    }

    pub(super) const fn is_full(&self) -> bool {
        !self.0[0].is_uninit() && !self.0[1].is_uninit()
    }

    pub(super) const fn push(&mut self, location: Location) -> Option<Location> {
        match &mut self.0 {
            [first, _] if first.is_uninit() => {
                *first = location;
                None
            }
            [_, second] if second.is_uninit() => {
                *second = location;
                None
            }
            [_, _] => Some(location),
        }
    }

    pub(crate) const fn inner(&self) -> &[Location; 2] {
        &self.0
    }
}

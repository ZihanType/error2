use crate::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct DoubleLocations([Location; 2]);

impl DoubleLocations {
    pub(super) const fn new(location: Location) -> Self {
        debug_assert!(!location.is_uninit());
        Self([location, Location::uninit()])
    }

    pub(super) const fn is_full(&self) -> bool {
        let [first, second] = &self.0;
        debug_assert!(!first.is_uninit());

        !second.is_uninit()
    }

    pub(super) const fn push(&mut self, location: Location) -> Option<Location> {
        let [first, second] = &mut self.0;
        debug_assert!(!first.is_uninit());

        if second.is_uninit() {
            *second = location;
            None
        } else {
            Some(location)
        }
    }

    pub(crate) const fn inner(&self) -> &[Location; 2] {
        &self.0
    }
}

mod double_locations;
mod message;

use std::{any, error::Error, fmt, mem};

use self::{double_locations::DoubleLocations, message::Message};
use crate::Location;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum BakctraceEntry {
    Message(Message),
    Locations(DoubleLocations),
}

const _: () = {
    ["Size of `Message`"][mem::size_of::<Message>() - 24usize];
    ["Size of `DoubleLocations`"][mem::size_of::<DoubleLocations>() - 24usize];
    ["`Message` and `DoubleLocations` must have the same size"]
        [mem::size_of::<Message>() - mem::size_of::<DoubleLocations>()];
    ["Size of `BakctraceEntry`"][mem::size_of::<BakctraceEntry>() - 32usize];
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Backtrace {
    entries: Vec<BakctraceEntry>,
}

impl fmt::Debug for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Backtrace {{ ... }}")
    }
}

impl Backtrace {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn with_head<E: Error>(source: &E) -> Backtrace {
        fn inner(type_name: &'static str, display: String) -> Backtrace {
            Backtrace {
                entries: vec![BakctraceEntry::Message(Message::new(type_name, display))],
            }
        }

        let type_name = any::type_name::<E>();
        let display = source.to_string();

        inner(type_name, display)
    }

    pub(crate) fn push_error(
        &mut self,
        type_name: &'static str,
        display: String,
        location: Location,
    ) {
        self.entries
            .push(BakctraceEntry::Message(Message::new(type_name, display)));

        self.entries
            .push(BakctraceEntry::Locations(DoubleLocations::new(location)));
    }

    pub(crate) const fn head_and_entries(&self) -> (Option<&Message>, &[BakctraceEntry]) {
        let entries = self.entries.as_slice();

        match entries {
            [] => (None, &[]),
            [BakctraceEntry::Locations(_), ..] => unreachable!(),
            [BakctraceEntry::Message(first), rest @ ..] => match rest {
                [] => (Some(first), &[]),
                [BakctraceEntry::Message(_second), ..] => (Some(first), rest),
                [BakctraceEntry::Locations(_second), ..] => (None, entries),
            },
        }
    }

    pub(crate) fn push_location(&mut self, location: Location) {
        debug_assert!(matches!(
            self.entries.first(),
            Some(BakctraceEntry::Message(_))
        ));

        let entry = self
            .entries
            .last_mut()
            .expect("there is must at least one message entry");

        match entry {
            BakctraceEntry::Locations(locations) if !locations.is_full() => {
                let l = locations.push(location);
                debug_assert!(l.is_none());
            }
            BakctraceEntry::Message(_) | BakctraceEntry::Locations(_) => {
                self.entries
                    .push(BakctraceEntry::Locations(DoubleLocations::new(location)));
            }
        }
    }

    #[inline]
    pub(crate) fn take(&mut self) -> Self {
        Self {
            entries: mem::take(&mut self.entries),
        }
    }
}

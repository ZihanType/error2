use std::{fmt, mem};

use crate::{Location, StaticStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Message {
    type_name: StaticStr,
    display: Box<str>,
    index: usize,
}

impl Message {
    pub(crate) const fn type_name(&self) -> &StaticStr {
        &self.type_name
    }

    pub(crate) const fn display(&self) -> &str {
        &self.display
    }

    pub(crate) const fn index(&self) -> usize {
        self.index
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Head {
    type_name: StaticStr,
    display: Box<str>,
}

impl Head {
    pub(crate) const fn type_name(&self) -> &StaticStr {
        &self.type_name
    }

    pub(crate) const fn display(&self) -> &str {
        &self.display
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Backtrace {
    head: Option<Head>,
    messages: Vec<Message>,
    locations: Vec<Location>,
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
            head: None,
            messages: Vec::new(),
            locations: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn with_head(type_name: &'static str, display: String) -> Self {
        Self {
            head: Some(Head {
                type_name: type_name.into(),
                display: display.into(),
            }),
            messages: Vec::new(),
            locations: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn push_error(&mut self, type_name: &'static str, display: String, location: Location) {
        self.messages.push(Message {
            type_name: type_name.into(),
            display: display.into(),
            index: self.locations.len(),
        });

        self.locations.push(location);
    }

    pub(crate) const fn head(&self) -> &Option<Head> {
        &self.head
    }

    pub(crate) const fn messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub(crate) const fn locations(&self) -> &Vec<Location> {
        &self.locations
    }

    #[inline]
    pub(crate) fn attach_location(&mut self, location: Location) {
        self.locations.push(location);
    }

    pub(crate) fn take(&mut self) -> Backtrace {
        Backtrace {
            head: self.head.take(),
            messages: mem::take(&mut self.messages),
            locations: mem::take(&mut self.locations),
        }
    }
}

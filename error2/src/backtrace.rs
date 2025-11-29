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

    pub(crate) const fn is_head(&self) -> bool {
        self.index == usize::MAX
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Backtrace {
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
            messages: Vec::new(),
            locations: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn with_head(type_name: &'static str, display: String) -> Self {
        Self {
            messages: vec![Message {
                type_name: type_name.into(),
                display: display.into(),
                index: usize::MAX,
            }],
            locations: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn push_error(&mut self, type_name: &'static str, display: String, location: Location) {
        let index = self.locations.len();

        self.messages.push(Message {
            type_name: type_name.into(),
            display: display.into(),
            index,
        });

        self.push_location(location);
    }

    pub(crate) const fn head_and_messages(&self) -> (Option<&Message>, &[Message]) {
        let messages = self.messages.as_slice();

        match messages {
            [] => (None, &[]),
            [first, rest @ ..] if first.is_head() => (Some(first), rest),
            _ => (None, messages),
        }
    }

    pub(crate) const fn locations(&self) -> &Vec<Location> {
        &self.locations
    }

    const fn has_head(&self) -> bool {
        matches!(self.messages.as_slice().first(), Some(msg) if msg.is_head())
    }

    #[inline]
    pub(crate) fn push_location(&mut self, location: Location) {
        debug_assert!(
            if self.has_head() {
                self.messages.len() > 1
            } else {
                !self.messages.is_empty()
            }
        );

        self.locations.push(location);
    }

    pub(crate) fn take(&mut self) -> Backtrace {
        Backtrace {
            messages: mem::take(&mut self.messages),
            locations: mem::take(&mut self.locations),
        }
    }
}

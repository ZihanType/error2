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

/// A backtrace that tracks error propagation through the call stack.
///
/// `Backtrace` stores a complete history of an error's journey, including:
/// - Error messages and type names
/// - Source code locations (file:line:column)
/// - Nested error chains
///
/// # Overview
///
/// Unlike `std::backtrace::Backtrace` which captures the call stack,
/// `error2::Backtrace` tracks the logical error propagation path through
/// your application, showing where errors were created, converted, and propagated.
///
/// # Creation
///
/// Backtraces are typically created automatically by the `#[derive(Error2)]` macro:
///
/// ```
/// use error2::prelude::*;
///
/// #[derive(Debug, Error2)]
/// #[error2(display("my error"))]
/// struct MyError {
///     backtrace: Backtrace,
/// }
/// ```
///
/// # Accessing Error Information
///
/// Use `.error_message()` to get a formatted error chain:
///
/// ```
/// # use error2::prelude::*;
/// # use std::io;
/// # #[derive(Debug, Error2)]
/// # #[error2(display("test error"))]
/// # struct TestError { source: io::Error, backtrace: Backtrace }
/// # fn operation() -> Result<(), TestError> {
/// #     let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
/// #     Err(err).context(TestError2)
/// # }
/// use regex::Regex;
///
/// if let Err(e) = operation() {
///     let msg = e.backtrace().error_message();
///
///     // Full error format with location tracking:
///     // TestError: test error
///     //     at /path/to/file.rs:128:14
///     // std::io::error::Error: file not found
///
///     let re = Regex::new(concat!(
///         r"(?s)^.+TestError: test error",
///         r"\n    at .+\.rs:\d+:\d+",
///         r"\nstd::io::error::Error: file not found$",
///     ))
///     .unwrap();
///     assert!(re.is_match(msg.as_ref()));
/// }
/// ```
///
/// # Location Tracking
///
/// The backtrace automatically records the location where an error is first created.
/// To track propagation through your call stack, you must manually call:
/// - `.attach()` - Records the current location when an error passes through
/// - `.attach_location()` - Records a specific location
///
/// When converting errors:
/// - `.context()` automatically captures where the conversion happens
/// - `.build()` / `.fail()` automatically captures where the root error is created
///
/// These methods use `#[track_caller]` to capture the caller's location without manual intervention.
///
/// # Example with Nested Errors
///
/// ```
/// use std::io;
///
/// use error2::prelude::*;
/// use regex::Regex;
///
/// #[derive(Debug, Error2)]
/// pub enum ConfigError {
///     #[error2(display("Failed to read: {path}"))]
///     Read {
///         path: String,
///         source: io::Error,
///         backtrace: Backtrace,
///     },
/// }
///
/// #[derive(Debug, Error2)]
/// pub enum AppError {
///     #[error2(display("Config error"))]
///     Config { source: ConfigError },
/// }
///
/// fn read_config() -> Result<String, ConfigError> {
///     std::fs::read_to_string("config.toml").context(Read2 {
///         path: "config.toml",
///     })
/// }
///
/// fn start_app() -> Result<String, AppError> {
///     read_config().context(Config2)
/// }
///
/// if let Err(e) = start_app() {
///     let msg = e.backtrace().error_message();
///
///     // Full error chain format:
///     // AppError: Config error
///     //     at /path/to/file.rs:184:19
///     // ConfigError: Failed to read: config.toml
///     //     at /path/to/file.rs:178:44
///     // std::io::error::Error: No such file or directory (os error 2)
///
///     let re = Regex::new(concat!(
///         r"(?s)^.+AppError: Config error",
///         r"\n    at .+\.rs:\d+:\d+",
///         r"\n.+ConfigError: Failed to read: config\.toml",
///         r"\n    at .+\.rs:\d+:\d+",
///         r"\nstd::io::error::Error: No such file or directory \(os error 2\)$",
///     ))
///     .unwrap();
///     assert!(re.is_match(msg.as_ref()));
/// }
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Backtrace {
    entries: Vec<BakctraceEntry>,
}

impl fmt::Debug for Backtrace {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Backtrace {{ ... }}")
    }
}

impl Backtrace {
    #[doc(hidden)]
    #[inline]
    pub const fn new() -> Self {
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

    /// Returns a formatted string containing the complete error chain.
    ///
    /// This method produces a human-readable representation of the entire error
    /// history, including all nested errors and their locations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use error2::prelude::*;
    /// # use std::io;
    /// # #[derive(Debug, Error2)]
    /// # #[error2(display("test error"))]
    /// # struct TestError { source: io::Error, backtrace: Backtrace }
    /// # fn operation() -> Result<(), TestError> {
    /// #     let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// #     Err(err).context(TestError2)
    /// # }
    /// use regex::Regex;
    ///
    /// if let Err(e) = operation() {
    ///     let msg = e.backtrace().error_message();
    ///
    ///     // Full error format with location tracking:
    ///     // TestError: test error
    ///     //     at /path/to/file.rs:197:14
    ///     // std::io::error::Error: file not found
    ///
    ///     let re = Regex::new(concat!(
    ///         r"(?s)^.+TestError: test error",
    ///         r"\n    at .+\.rs:\d+:\d+",
    ///         r"\nstd::io::error::Error: file not found$",
    ///     ))
    ///     .unwrap();
    ///     assert!(re.is_match(msg.as_ref()));
    /// }
    /// ```
    pub fn error_message(&self) -> Box<str> {
        crate::extract_error_message(self)
    }
}

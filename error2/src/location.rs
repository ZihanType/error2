use std::{fmt, panic};

use crate::FilePath;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Location {
    file: FilePath,
    line: u32,
    column: u32,
}

impl Location {
    #[doc(hidden)]
    #[inline]
    pub fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }

    #[track_caller]
    #[inline]
    pub fn caller() -> Self {
        Self::from_std(panic::Location::caller())
    }

    #[inline]
    pub fn file(&self) -> &'static str {
        self.file.into()
    }

    #[inline]
    pub const fn line(&self) -> u32 {
        self.line
    }

    #[inline]
    pub const fn column(&self) -> u32 {
        self.column
    }

    #[inline]
    pub fn from_std(location: &'static panic::Location<'_>) -> Self {
        Self {
            file: location.file().into(),
            line: location.line(),
            column: location.column(),
        }
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "snafu")))]
    #[cfg(feature = "snafu")]
    #[inline]
    pub fn from_snafu(location: snafu::Location) -> Self {
        Self {
            file: location.file.into(),
            line: location.line,
            column: location.column,
        }
    }
}

impl fmt::Display for Location {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl<'a> From<&'static panic::Location<'a>> for Location {
    #[inline]
    fn from(location: &'static panic::Location<'a>) -> Self {
        Self::from_std(location)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "snafu")))]
#[cfg(feature = "snafu")]
impl From<snafu::Location> for Location {
    #[inline]
    fn from(location: snafu::Location) -> Self {
        Self::from_snafu(location)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "snafu")))]
#[cfg(feature = "snafu")]
impl From<Location> for snafu::Location {
    #[inline]
    fn from(location: Location) -> Self {
        snafu::Location::new(location.file.into(), location.line, location.column)
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::*;

    #[test]
    fn test_the_effects_of_tracker_caller() {
        let Tuple {
            from_std: loc_from_std_by_fn,
            from_crate: loc_from_crate_by_fn,
        } = location_by_fn();

        assert_eq!(loc_from_std_by_fn, loc_from_crate_by_fn);

        let Tuple {
            from_std: loc_from_std_by_macro,
            from_crate: loc_from_crate_by_macro,
        } = location_by_macro();

        assert_ne!(loc_from_std_by_macro, loc_from_crate_by_macro);

        assert_ne!(loc_from_std_by_fn, loc_from_std_by_macro);
        assert_ne!(loc_from_std_by_fn, loc_from_crate_by_macro);
    }

    struct Tuple {
        from_std: Location,
        from_crate: Location,
    }

    #[track_caller]
    fn location_by_fn() -> Tuple {
        let from_std = Location::from_std(panic::Location::caller());
        let from_crate = Location::caller();

        Tuple {
            from_std,
            from_crate,
        }
    }

    #[track_caller]
    fn location_by_macro() -> Tuple {
        let from_std = Location {
            file: file!().into(),
            line: line!(),
            column: column!(),
        };

        let from_crate = crate::location!();

        Tuple {
            from_std,
            from_crate,
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_deserialize_locations() {
        macro_rules! location {
            ($file:literal) => {
                Location {
                    file: $file.into(),
                    line: line!(),
                    column: column!(),
                }
            };
        }

        let origin = vec![
            location!("你好，世界"),
            location!("Hello World"),
            location!("Bonjour le monde"),
            location!("Hola Mundo"),
            location!("Hallo Welt"),
            location!("Ciao Mondo"),
            location!("Привет мир"),
            location!("こんにちは世界"),
            location!("안녕하세요 세계"),
            location!("مرحبا بالعالم"),
            location!("שלום עולם"),
            location!("Γειά σου Κόσμε"),
        ];

        let deserialized = {
            let serialized = serde_json::to_vec(&origin).unwrap();
            serde_json::from_slice::<Vec<Location>>(&serialized).unwrap()
        };

        assert_eq!(origin, deserialized);
    }
}

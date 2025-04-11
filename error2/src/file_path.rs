use std::{fmt, sync::LazyLock};

use crate::{Id, Interner};

static INTERNER: LazyLock<Interner> = LazyLock::new(Interner::default);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FilePath(Id);

impl fmt::Debug for FilePath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = INTERNER.lookup(self.0);
        fmt::Debug::fmt(s, f)
    }
}

impl fmt::Display for FilePath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = INTERNER.lookup(self.0);
        fmt::Display::fmt(s, f)
    }
}

impl From<&'static str> for FilePath {
    #[inline]
    fn from(s: &'static str) -> Self {
        let id = INTERNER.intern_static(s);
        FilePath(id)
    }
}

impl From<FilePath> for &'static str {
    #[inline]
    fn from(FilePath(id): FilePath) -> Self {
        INTERNER.lookup(id)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl serde::Serialize for FilePath {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = INTERNER.lookup(self.0);
        serializer.serialize_str(s)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FilePath {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let id = INTERNER.intern_normal(s);
        Ok(FilePath(id))
    }
}

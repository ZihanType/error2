mod interner;
mod small_string;

use std::{fmt, sync::LazyLock};

use self::interner::{Id, Interner};

static INTERNER: LazyLock<Interner> = LazyLock::new(Interner::default);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StaticStr(Id);

impl fmt::Debug for StaticStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = INTERNER.lookup(self.0);
        fmt::Debug::fmt(s, f)
    }
}

impl fmt::Display for StaticStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = INTERNER.lookup(self.0);
        fmt::Display::fmt(s, f)
    }
}

impl From<&'static str> for StaticStr {
    #[inline]
    fn from(s: &'static str) -> Self {
        let id = INTERNER.intern_static(s);
        StaticStr(id)
    }
}

impl From<StaticStr> for &'static str {
    #[inline]
    fn from(StaticStr(id): StaticStr) -> Self {
        INTERNER.lookup(id)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl serde::Serialize for StaticStr {
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
impl<'de> serde::Deserialize<'de> for StaticStr {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let id = INTERNER.intern_normal(s);
        Ok(StaticStr(id))
    }
}

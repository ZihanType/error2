mod interner;
mod small_string;

use std::{fmt, sync::LazyLock};

use self::interner::Interner;

static INTERNER: LazyLock<Interner> = LazyLock::new(Interner::default);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StrId(u32);

impl StrId {
    pub(crate) const fn new(id: usize) -> Self {
        assert!(id < (u32::MAX as usize));
        StrId(id as u32)
    }

    pub(crate) const fn uninit() -> Self {
        StrId(u32::MAX)
    }

    pub(crate) const fn is_uninit(&self) -> bool {
        self.0 == u32::MAX
    }

    pub(crate) const fn inner(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for StrId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = INTERNER.lookup(self);
        fmt::Debug::fmt(s, f)
    }
}

impl fmt::Display for StrId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = INTERNER.lookup(self);
        fmt::Display::fmt(s, f)
    }
}

impl From<&'static str> for StrId {
    #[inline]
    fn from(s: &'static str) -> Self {
        INTERNER.intern_static(s)
    }
}

impl From<StrId> for &'static str {
    #[inline]
    fn from(id: StrId) -> Self {
        INTERNER.lookup(&id)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl serde::Serialize for StrId {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = INTERNER.lookup(self);
        serializer.serialize_str(s)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StrId {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let id = INTERNER.intern_normal(s);
        Ok(id)
    }
}

use std::{
    borrow::Borrow,
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug, Clone, Eq)]
pub(crate) enum SmallString {
    Borrowed(&'static str),

    #[allow(dead_code)]
    Owned(Arc<str>),
}

impl SmallString {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            SmallString::Borrowed(s) => s,
            SmallString::Owned(s) => s,
        }
    }
}

impl Borrow<str> for SmallString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for SmallString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for SmallString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Ord for SmallString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for SmallString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for SmallString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

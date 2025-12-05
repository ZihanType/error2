use std::sync::Arc;

use append_only_vec::AppendOnlyVec;
use scc::HashIndex;

use super::{StrId, small_string::SmallString};

#[derive(Debug, Default)]
pub(super) struct Interner {
    vec: AppendOnlyVec<SmallString>,
    map: HashIndex<SmallString, StrId>,
}

impl Interner {
    pub(super) fn intern_static(&self, s: &'static str) -> StrId {
        *self
            .map
            .entry_sync(SmallString::Borrowed(s))
            .or_insert_with(|| {
                let id = StrId::new(self.vec.len());
                self.vec.push(SmallString::Borrowed(s));
                id
            })
    }

    #[allow(dead_code)]
    pub(super) fn intern_normal(&self, s: &str) -> StrId {
        if let Some(id) = self.map.get_sync(s) {
            return *id.get();
        }

        let id = StrId::new(self.vec.len());

        let s: Arc<str> = s.into();
        self.vec.push(SmallString::Owned(s.clone()));
        self.map
            .insert_sync(SmallString::Owned(s), id)
            .expect("unreachable, if we have a collision, we should have found it before");

        id
    }

    #[inline]
    pub(super) fn lookup(&self, id: &StrId) -> &str {
        self.vec[id.inner()].as_str()
    }
}

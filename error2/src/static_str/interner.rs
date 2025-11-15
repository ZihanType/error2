use std::sync::Arc;

use append_only_vec::AppendOnlyVec;
use scc::HashIndex;

use super::small_string::SmallString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Id(u32);

#[derive(Debug, Default)]
pub(super) struct Interner {
    vec: AppendOnlyVec<SmallString>,
    map: HashIndex<SmallString, Id>,
}

impl Interner {
    pub(super) fn intern_static(&self, s: &'static str) -> Id {
        *self
            .map
            .entry_sync(SmallString::Borrowed(s))
            .or_insert_with(|| {
                let id = self.vec.len();
                assert!(id <= u32::MAX as usize);
                let id = Id(id as u32);

                self.vec.push(SmallString::Borrowed(s));

                id
            })
    }

    #[allow(dead_code)]
    pub(super) fn intern_normal(&self, s: &str) -> Id {
        if let Some(id) = self.map.get_sync(s) {
            return *id.get();
        }

        let s: Arc<str> = s.into();

        let id = self.vec.len();
        assert!(id <= u32::MAX as usize);
        let id = Id(id as u32);

        self.vec.push(SmallString::Owned(s.clone()));
        self.map
            .insert_sync(SmallString::Owned(s), id)
            .expect("unreachable, if we have a collision, we should have found it before");

        id
    }

    #[inline]
    pub(super) fn lookup(&self, Id(id): Id) -> &str {
        self.vec[id as usize].as_str()
    }
}

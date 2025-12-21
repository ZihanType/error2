use std::marker::PhantomData;

use crate::{Attach, Location};

impl<T, W> Attach<AttachIter<Self, W>> for T
where
    T: Iterator,
    T::Item: Attach<W>,
{
    #[inline]
    fn attach_location(self, location: Location) -> AttachIter<Self, W> {
        AttachIter {
            inner: self,
            location,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct AttachIter<I, W> {
    inner: I,
    location: Location,
    phantom: PhantomData<W>,
}

impl<I, W> Iterator for AttachIter<I, W>
where
    I: Iterator,
    I::Item: Attach<W>,
{
    type Item = W;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(item) => Some(item.attach_location(self.location)),
            None => None,
        }
    }
}

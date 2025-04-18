use crate::{Attach, Location};

pub trait IteratorExt: Iterator + Sized
where
    Self::Item: Attach,
{
    #[track_caller]
    #[inline]
    fn attach(self) -> AttachIter<Self> {
        AttachIter {
            inner: self,
            location: Location::caller(),
        }
    }

    #[inline]
    fn attach_location(self, location: Location) -> AttachIter<Self> {
        AttachIter {
            inner: self,
            location,
        }
    }
}

impl<T: Iterator> IteratorExt for T where T::Item: Attach {}

#[derive(Debug, Clone, Copy)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct AttachIter<I> {
    inner: I,
    location: Location,
}

impl<I> Iterator for AttachIter<I>
where
    I: Iterator,
    I::Item: Attach,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(item) => Some(item.attach_location(self.location)),
            None => None,
        }
    }
}

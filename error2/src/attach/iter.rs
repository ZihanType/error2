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

/// Iterator adapter that attaches location to each error.
///
/// # Example
///
/// ```
/// use error2::prelude::*;
///
/// # fn example() -> Result<(), BoxedError2> {
/// let results: Vec<Result<i32, BoxedError2>> = vec![Ok(1), Ok(2)];
/// let results = results.into_iter().attach(); // Error will include this location
///
/// for value in results {
///     let v = value?;
///     println!("{}", v);
/// }
/// # Ok(())
/// # }
/// ```
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

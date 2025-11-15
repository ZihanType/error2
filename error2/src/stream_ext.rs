use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::{Attach, Location};

pub trait StreamExt: Stream + Sized
where
    Self::Item: Attach,
{
    #[track_caller]
    #[inline]
    fn attach(self) -> AttachStream<Self> {
        self.attach_location(Location::caller())
    }

    #[inline]
    fn attach_location(self, location: Location) -> AttachStream<Self> {
        AttachStream {
            inner: self,
            location,
        }
    }
}

impl<T: Stream> StreamExt for T where T::Item: Attach {}

pin_project! {
    #[derive(Debug, Clone, Copy)]
    #[must_use = "streams do nothing unless polled"]
    pub struct AttachStream<S> {
        #[pin]
        inner: S,
        location: Location,
    }
}

impl<S> Stream for AttachStream<S>
where
    S: Stream,
    S::Item: Attach,
{
    type Item = S::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.inner.poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item.attach_location(*this.location))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

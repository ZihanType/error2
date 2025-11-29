use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::{Attach, Location};

impl<T, W> Attach<AttachStream<Self, W>> for T
where
    T: Stream,
    T::Item: Attach<W>,
{
    #[inline]
    fn attach_location(self, location: Location) -> AttachStream<Self, W> {
        AttachStream {
            inner: self,
            location,
            phantom: PhantomData,
        }
    }
}

pin_project! {
    #[derive(Debug, Clone, Copy)]
    #[must_use = "streams do nothing unless polled"]
    pub struct AttachStream<S, W> {
        #[pin]
        inner: S,
        location: Location,
        phantom: PhantomData<W>,
    }
}

impl<S, W> Stream for AttachStream<S, W>
where
    S: Stream,
    S::Item: Attach<W>,
{
    type Item = W;

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

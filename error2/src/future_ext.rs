use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{Attach, Location};

pub trait FutureExt: Future + Sized
where
    Self::Output: Attach,
{
    #[track_caller]
    #[inline]
    fn attach(self) -> AttachFuture<Self> {
        self.attach_location(Location::caller())
    }

    #[inline]
    fn attach_location(self, location: Location) -> AttachFuture<Self> {
        AttachFuture {
            inner: self,
            location,
        }
    }
}

impl<T: Future> FutureExt for T where T::Output: Attach {}

pin_project! {
    #[derive(Debug, Clone, Copy)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct AttachFuture<F> {
        #[pin]
        inner: F,
        location: Location,
    }
}

impl<F> Future for AttachFuture<F>
where
    F: Future,
    F::Output: Attach,
{
    type Output = F::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(output) => Poll::Ready(output.attach_location(*this.location)),
            Poll::Pending => Poll::Pending,
        }
    }
}

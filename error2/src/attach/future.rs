use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{Attach, Location};

impl<T, W> Attach<AttachFuture<Self, W>> for T
where
    T: Future,
    T::Output: Attach<W>,
{
    #[inline]
    fn attach_location(self, location: Location) -> AttachFuture<Self, W> {
        AttachFuture {
            inner: self,
            location,
            phantom: PhantomData,
        }
    }
}

pin_project! {
    /// Future adapter that attaches location to errors.
    ///
    /// # Example
    ///
    /// ```
    /// use error2::prelude::*;
    /// use std::future::Future;
    /// use std::pin::Pin;
    ///
    /// # async fn async_operation() -> Result<(), BoxedError2> {
    /// #     Ok(())
    /// # }
    /// #
    /// async fn process() -> Result<(), BoxedError2> {
    ///     async_operation()
    ///     .attach() // Location recorded here
    ///     .await
    /// }
    /// ```
    #[derive(Debug, Clone, Copy)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct AttachFuture<F, W> {
        #[pin]
        inner: F,
        location: Location,
        phantom: PhantomData<W>,
    }
}

impl<F, W> Future for AttachFuture<F, W>
where
    F: Future,
    F::Output: Attach<W>,
{
    type Output = W;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(output) => Poll::Ready(output.attach_location(*this.location)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
mod future;
mod iter;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
mod stream;

#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
pub use self::future::AttachFuture;
pub use self::iter::AttachIter;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub use self::stream::AttachStream;

#![cfg_attr(docsrs, feature(doc_cfg))]

mod attach;
mod error_ext;
mod extract;
mod file_path;
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
mod future_ext;
mod interner;
mod iterator_ext;
mod location;
mod locations;
mod macros;
mod next_error;
mod small_string;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
mod stream_ext;

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use error2_derive::ErrorExt;

#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
pub use self::future_ext::{AttachFuture, FutureExt};
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub use self::stream_ext::{AttachStream, StreamExt};
pub use self::{
    attach::Attach,
    error_ext::ErrorExt,
    extract::extract_error_stack,
    iterator_ext::{AttachIter, IteratorExt},
    location::Location,
    locations::Locations,
    next_error::NextError,
};
pub(crate) use self::{
    file_path::FilePath,
    interner::{Id, Interner},
    small_string::SmallString,
};

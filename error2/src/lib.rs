#![cfg_attr(docsrs, feature(doc_cfg))]

mod attach;
mod backtrace;
mod boxed;
mod context;
mod error2;
mod extract;
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
mod future;
mod iter;
mod location;
mod macros;
mod root_error;
mod str_id;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
mod stream;
mod transform;

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use ::error2_derive::Error2;

#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
pub use self::future::AttachFuture;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub use self::stream::AttachStream;
pub use self::{
    attach::Attach,
    backtrace::Backtrace,
    boxed::{BoxedError2, ViaErr2, ViaRoot, ViaStd},
    context::Context,
    error2::Error2,
    iter::AttachIter,
    location::Location,
    root_error::RootError,
    transform::{MiddleToTarget, SourceToTarget},
};
pub(crate) use self::{backtrace::BakctraceEntry, extract::extract_error_message, str_id::StrId};

pub(crate) mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaPartial {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaFull {}
}

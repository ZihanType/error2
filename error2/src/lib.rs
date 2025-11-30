#![cfg_attr(docsrs, feature(doc_cfg))]

mod attach;
mod backtrace;
mod boxed;
mod error2;
mod error_wrap;
mod extract;
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
mod future;
mod iter;
mod location;
mod macros;
mod option_ext;
mod result_ext;
mod static_str;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
mod stream;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use ::static_assertions::assert_not_impl_any;
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use error2_derive::Error2;

#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
#[cfg(feature = "future")]
pub use self::future::AttachFuture;
pub(crate) use self::static_str::StaticStr;
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
#[cfg(feature = "stream")]
pub use self::stream::AttachStream;
pub use self::{
    attach::Attach,
    backtrace::Backtrace,
    boxed::{BoxedError2, ViaErr2, ViaNone, ViaStd},
    error_wrap::ErrorWrap,
    error2::Error2,
    extract::{extract_error_message, extract_error_stack},
    iter::AttachIter,
    location::Location,
    option_ext::{NoneError, OptionExt},
    result_ext::ResultExt,
};

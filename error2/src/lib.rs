#![cfg_attr(docsrs, feature(doc_cfg))]

mod _attach;
mod backtrace;
mod boxed;
mod context;
mod error2;
mod extract;
mod location;
mod macros;
mod root_error;
mod str_id;

pub mod attach;
pub mod kind;
pub mod transform;

pub mod prelude {
    #[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
    #[cfg(feature = "derive")]
    pub use ::error2_derive::Error2;

    // traits
    pub use crate::{Attach as _, Context as _, RootError as _, error2::Error2};
    // types
    pub use crate::{Backtrace, BoxedError2, ViaErr2, ViaRoot, ViaStd};
}

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use ::error2_derive::Error2;

pub use self::{
    _attach::Attach,
    backtrace::Backtrace,
    boxed::{BoxedError2, ViaErr2, ViaRoot, ViaStd},
    context::Context,
    error2::Error2,
    location::Location,
    root_error::RootError,
};
pub(crate) use self::{backtrace::BakctraceEntry, extract::extract_error_message, str_id::StrId};

pub(crate) mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaPartial {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaFull {}
}

#[doc(hidden)]
pub fn push_error<E: Error2 + ?Sized>(error: &mut E, location: Location) {
    let display = error.to_string();
    let backtrace = error.backtrace_mut();
    let type_name = core::any::type_name::<E>();

    backtrace.push_error(type_name, display, location);
}

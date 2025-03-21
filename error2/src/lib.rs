#![cfg_attr(docsrs, feature(doc_cfg))]

mod attach;
mod error_ext;
mod extract;
mod location;
mod locations;
mod macros;
mod next_error;

#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use error2_derive::ErrorExt;

pub use self::{
    attach::Attach, error_ext::ErrorExt, extract::extract_error_stack, location::Location,
    locations::Locations, next_error::NextError,
};

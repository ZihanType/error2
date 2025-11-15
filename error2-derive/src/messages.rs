use syn::Ident;

use crate::types::ContextRefClass;

pub(crate) const SUPPORTED_TYPES: &str =
    "`Error2` can only be derived for structs and enums with named fields";

pub(crate) const AT_LEAST_ONE_FIELD: &str = "must have at least one field";

pub(crate) const AT_LEAST_ONE_VARIANT: &str = "must have at least one variant";

pub(crate) const EXPECTED_IDENT: &str = "expected identifier";

pub(crate) const DISPLAY_MUST_IN_META_LIST: &str = "`display` attribute can only appear in meta list, such as `#[error2(display(\"some message {}\", some_field))]` or `#[error2(display(false))]`";

pub(crate) const VIS_MUST_IN_META_LIST: &str =
    "`vis` attribute can only appear in meta list, such as `#[error2(vis(pub))]`";

pub(crate) const DISABLE_DISPLAY_MUST_ON_TYPE: &str =
    "`#[error2(display(false))]` is not supported on variants, only on structs or enums";

pub(crate) const STD_MUST_IN_PATH: &str =
    "`std` attribute can only appear in path, such as `#[error2(std)]`";

pub(crate) const MODULE_MUST_IN_PATH: &str =
    "`module` attribute can only appear in path, such as `#[error2(module)]`";

pub(crate) const NO_DISPLAY_ON_STRUCT: &str = "no `display` attribute found on the struct, it must be specified, such as `#[error2(display(\"some message {}\", some_field))]` or `#[error2(display(false))]`";

pub(crate) const DISPLAY_TOKENS_NOT_ON_ENUM: &str =
    "enums can only omit the `display` attribute or use `#[error2(display(false))]`";

pub(crate) const NO_DISPLAY_ON_ENUM_OR_VARIANT: &str = "no `display` attribute found on either the enum or the variant, at least one must be specified";

pub(crate) const DISPLAY_SET_TWO_PLACE: &str = "`display` attribute specified multiple times, once on the enum (disabled) and once on the variant";

pub(crate) fn unknown_single_attr(path_ident: &Ident, attr: &'static str) -> String {
    format!(
        "unknown attribute `{}`, only `{}` is supported",
        path_ident, attr
    )
}

pub(crate) fn specified_multiple_times(attr: &'static str) -> String {
    format!("`{}` attribute specified multiple times", attr)
}

pub(crate) fn incorrect_leaf_error_def(class: ContextRefClass) -> String {
    format!(
        "this {} has neither source nor backtrace fields, it appears to be a leaf error, but leaf errors must have a `backtrace` field",
        class.as_str()
    )
}

pub(crate) fn incorrect_leaf_std_def(class: ContextRefClass) -> String {
    format!(
        "this {} has both source and backtrace fields, it appears the source field type only implements the `std::error::Error` trait rather than the `Error2` trait, therefore the `#[error2(std)]` attribute must be used",
        class.as_str()
    )
}

pub(crate) fn incorrect_leaf_err2_def(class: ContextRefClass) -> String {
    format!(
        "this {} has a source field but no backtrace field, it appears the source field type already implements the `Error2` trait rather than just the `std::error::Error` trait, therefore the `#[error2(std)]` attribute cannot be used",
        class.as_str()
    )
}

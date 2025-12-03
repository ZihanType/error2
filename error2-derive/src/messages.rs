use syn::Ident;

use crate::types::ContextKind;

pub(crate) const SUPPORTED_TYPES: &str =
    "`Error2` can only be derived for structs and enums with named fields";

pub(crate) const AT_LEAST_ONE_FIELD: &str = "must have at least one field";

pub(crate) const AT_LEAST_ONE_VARIANT: &str = "must have at least one variant";

pub(crate) const EXPECTED_IDENT: &str = "expected identifier";

pub(crate) const DISPLAY_MUST_IN_META_LIST: &str = "`display` attribute can only appear in meta list, such as `#[error2(display(\"some message {}\", some_field))]` or `#[error2(display(false))]`";

pub(crate) const VIS_MUST_IN_META_LIST: &str =
    "`vis` attribute can only appear in meta list, such as `#[error2(vis(pub))]`";

pub(crate) const MODULE_MUST_IN_PATH: &str =
    "`module` attribute can only appear in path, such as `#[error2(module)]`";

pub(crate) const DISPLAY_TOKENS_NOT_ON_ENUM: &str =
    "enums can only omit the `display` attribute or use `#[error2(display(false))]`";

pub(crate) const MISSING_DISPLAY_ON_VARIANT: &str = "missing `#[error2(display(...))]` attribute";

pub(crate) fn unknown_single_attr(path_ident: &Ident, attr: &'static str) -> String {
    format!(
        "unknown attribute `{}`, only `{}` is supported",
        path_ident, attr
    )
}

pub(crate) fn specified_multiple_times(attr: &'static str) -> String {
    format!("`{}` attribute specified multiple times", attr)
}

pub(crate) fn incorrect_def(kind: ContextKind) -> String {
    format!(
        "this {} has neither `source` nor `backtrace` fields. If it's a root error, it must contain a `backtrace` field; if it's from std, it must contain both `source` and `backtrace` fields; if it's from error2, it must contain a `source` field",
        kind.as_str()
    )
}

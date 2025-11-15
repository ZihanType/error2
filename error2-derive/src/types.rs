use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Field, Ident, Token, Visibility, punctuated::Punctuated};

#[derive(Clone, Copy)]
pub(crate) enum ErrorKind {
    Leaf,
    Std,
    Err2,
}

impl ErrorKind {
    pub(crate) fn is_leaf(&self) -> bool {
        matches!(self, ErrorKind::Leaf)
    }
}

pub(crate) enum TypeDisplayAttr {
    None,
    Disabled {
        meta_span: Span,
    },
    Enabled {
        meta_span: Span,
        tokens: TokenStream,
    },
}

impl TypeDisplayAttr {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, TypeDisplayAttr::None)
    }
}

pub(crate) enum VartiantDisplayAttr {
    None,
    Enabled {
        meta_span: Span,
        tokens: TokenStream,
    },
}

impl VartiantDisplayAttr {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, VartiantDisplayAttr::None)
    }
}

pub(crate) struct TypeAttr {
    pub(crate) display: TypeDisplayAttr,
    pub(crate) context_vis: Visibility,
    pub(crate) mod_vis: Option<Visibility>,
}

pub(crate) struct VariantAttr {
    pub(crate) display: VartiantDisplayAttr,
}

pub(crate) enum FromStd {
    No,
    Yes { meta_span: Span },
}

impl FromStd {
    pub(crate) fn is_yes(&self) -> bool {
        matches!(self, FromStd::Yes { .. })
    }
}

pub(crate) struct FieldAttr {
    pub(crate) from_std: FromStd,
}

pub(crate) struct MyVariant {
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) ident: Ident,
    pub(crate) named_fields: Punctuated<Field, Token![,]>,
}

#[derive(Clone, Copy)]
pub(crate) enum ContextRefClass {
    Struct,
    Variant,
}

impl ContextRefClass {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ContextRefClass::Struct => "struct",
            ContextRefClass::Variant => "variant",
        }
    }
}

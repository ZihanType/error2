use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Field, Ident, Token, Visibility, punctuated::Punctuated};

#[derive(Clone, Copy)]
pub(crate) enum ErrorKind {
    Root,
    Std,
    Err2,
}

impl ErrorKind {
    pub(crate) fn is_root(&self) -> bool {
        matches!(self, ErrorKind::Root)
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

pub(crate) struct MyVariant {
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) ident: Ident,
    pub(crate) named_fields: Punctuated<Field, Token![,]>,
}

#[derive(Clone, Copy)]
pub(crate) enum ContextKind {
    Struct,
    Variant,
}

impl ContextKind {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ContextKind::Struct => "struct",
            ContextKind::Variant => "variant",
        }
    }
}

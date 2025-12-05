use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
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

pub(crate) struct TypeAttr {
    pub(crate) display: TypeDisplayAttr,
    pub(crate) context_vis: Visibility,
    pub(crate) mod_vis: Option<Visibility>,
}

pub(crate) struct VariantAttr {
    pub(crate) display: Option<TokenStream>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Trait {
    Debug,
    Display,
}

impl ToTokens for Trait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self {
            Trait::Debug => quote! { ::core::fmt::Debug },
            Trait::Display => quote! { ::core::fmt::Display },
        };
        tokens.extend(ts);
    }
}

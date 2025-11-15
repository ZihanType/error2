use crate::{
    messages::{
        DISABLE_DISPLAY_MUST_ON_TYPE, DISPLAY_MUST_IN_META_LIST, EXPECTED_IDENT,
        MODULE_MUST_IN_PATH, STD_MUST_IN_PATH, VIS_MUST_IN_META_LIST, specified_multiple_times,
        unknown_single_attr,
    },
    types::{FieldAttr, FromStd, TypeAttr, TypeDisplayAttr, VariantAttr, VartiantDisplayAttr},
};
use quote::quote;
use syn::{Attribute, LitBool, Meta, Token, Visibility, punctuated::Punctuated, spanned::Spanned};

pub(crate) fn parse_type_attr(attrs: &[Attribute]) -> syn::Result<TypeAttr> {
    fn inner(
        attr: &Attribute,
        display: &mut TypeDisplayAttr,
        vis: &mut Option<Visibility>,
        module: &mut bool,
        errors: &mut Vec<syn::Error>,
    ) {
        if !attr.path().is_ident("error2") {
            return;
        }

        let nested = match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return;
            }
        };

        for meta in nested {
            let path = meta.path();

            let Some(path_ident) = path.get_ident() else {
                errors.push(syn::Error::new(path.span(), EXPECTED_IDENT));
                continue;
            };

            if path_ident == "display" {
                let list = match meta {
                    Meta::List(meta_list) => meta_list,
                    Meta::Path(_) | Meta::NameValue(_) => {
                        errors.push(syn::Error::new(meta.span(), DISPLAY_MUST_IN_META_LIST));
                        continue;
                    }
                };

                if !display.is_none() {
                    errors.push(syn::Error::new(
                        list.span(),
                        specified_multiple_times("display"),
                    ));
                    continue;
                }

                match list.parse_args() {
                    Ok(LitBool { value: false, .. }) => {
                        *display = TypeDisplayAttr::Disabled {
                            meta_span: list.span(),
                        }
                    }
                    _ => {
                        *display = TypeDisplayAttr::Enabled {
                            meta_span: list.span(),
                            tokens: list.tokens,
                        }
                    }
                }
            } else if path_ident == "vis" {
                let list = match meta {
                    Meta::List(meta_list) => meta_list,
                    Meta::Path(_) | Meta::NameValue(_) => {
                        errors.push(syn::Error::new(meta.span(), VIS_MUST_IN_META_LIST));
                        continue;
                    }
                };

                if vis.is_some() {
                    errors.push(syn::Error::new(
                        list.span(),
                        specified_multiple_times("vis"),
                    ));
                    continue;
                }

                match list.parse_args::<Visibility>() {
                    Ok(v) => *vis = Some(v),
                    Err(e) => {
                        errors.push(e);
                    }
                }
            } else if path_ident == "module" {
                let path = match meta {
                    Meta::Path(path) => path,
                    Meta::List(_) | Meta::NameValue(_) => {
                        errors.push(syn::Error::new(meta.span(), MODULE_MUST_IN_PATH));
                        continue;
                    }
                };

                if *module {
                    errors.push(syn::Error::new(
                        path.span(),
                        specified_multiple_times("module"),
                    ));
                    continue;
                } else {
                    *module = true;
                }
            } else {
                errors.push(syn::Error::new(
                    path_ident.span(),
                    format!(
                        "unknown attribute `{}`, only `display`, `vis` and `module` are supported",
                        path_ident
                    ),
                ));
            }
        }
    }

    let mut display = TypeDisplayAttr::None;
    let mut vis: Option<Visibility> = None;
    let mut module = false;

    let mut errors = Vec::new();

    attrs
        .iter()
        .for_each(|attr| inner(attr, &mut display, &mut vis, &mut module, &mut errors));

    if let Some(e) = errors.into_iter().reduce(|mut a, b| {
        a.combine(b);
        a
    }) {
        return Err(e);
    }

    let (context_vis, mod_vis) = match (vis, module) {
        (None, false) => (Visibility::Inherited, None),
        (None, true) => (
            syn::parse2::<Visibility>(quote! { pub(super) }).unwrap(),
            Some(Visibility::Inherited),
        ),
        (Some(vis), false) => (vis, None),
        (Some(vis), true) => (vis.clone(), Some(vis)),
    };

    Ok(TypeAttr {
        display,
        context_vis,
        mod_vis,
    })
}

pub(crate) fn parse_variant_attr(attrs: &[Attribute]) -> syn::Result<VariantAttr> {
    fn inner(attr: &Attribute, display: &mut VartiantDisplayAttr, errors: &mut Vec<syn::Error>) {
        if !attr.path().is_ident("error2") {
            return;
        }

        let nested = match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return;
            }
        };

        for meta in nested {
            let path = meta.path();

            let Some(path_ident) = path.get_ident() else {
                errors.push(syn::Error::new(path.span(), EXPECTED_IDENT));
                continue;
            };

            if path_ident == "display" {
                let list = match meta {
                    Meta::List(meta_list) => meta_list,
                    Meta::Path(_) | Meta::NameValue(_) => {
                        errors.push(syn::Error::new(meta.span(), DISPLAY_MUST_IN_META_LIST));
                        continue;
                    }
                };

                if !display.is_none() {
                    errors.push(syn::Error::new(
                        list.span(),
                        specified_multiple_times("display"),
                    ));
                    continue;
                }

                match list.parse_args() {
                    Ok(LitBool { value: false, .. }) => {
                        errors.push(syn::Error::new(list.span(), DISABLE_DISPLAY_MUST_ON_TYPE));
                    }
                    _ => {
                        *display = VartiantDisplayAttr::Enabled {
                            meta_span: list.span(),
                            tokens: list.tokens,
                        }
                    }
                }
            } else {
                errors.push(syn::Error::new(
                    path_ident.span(),
                    unknown_single_attr(path_ident, "display"),
                ));
            }
        }
    }

    let mut display = VartiantDisplayAttr::None;
    let mut errors = Vec::new();

    attrs
        .iter()
        .for_each(|attr| inner(attr, &mut display, &mut errors));

    if let Some(e) = errors.into_iter().reduce(|mut a, b| {
        a.combine(b);
        a
    }) {
        return Err(e);
    }

    Ok(VariantAttr { display })
}

pub(crate) fn parse_field_attr(attrs: &[Attribute]) -> syn::Result<FieldAttr> {
    fn inner(attr: &Attribute, from_std: &mut FromStd, errors: &mut Vec<syn::Error>) {
        if !attr.path().is_ident("error2") {
            return;
        }

        let nested = match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return;
            }
        };

        for meta in nested {
            let path = meta.path();

            let Some(path_ident) = path.get_ident() else {
                errors.push(syn::Error::new(path.span(), EXPECTED_IDENT));
                continue;
            };

            if path_ident == "std" {
                let path = match meta {
                    Meta::Path(path) => path,
                    Meta::List(_) | Meta::NameValue(_) => {
                        errors.push(syn::Error::new(meta.span(), STD_MUST_IN_PATH));
                        continue;
                    }
                };

                if from_std.is_yes() {
                    errors.push(syn::Error::new(
                        path.span(),
                        specified_multiple_times("std"),
                    ));
                    continue;
                } else {
                    *from_std = FromStd::Yes {
                        meta_span: path.span(),
                    };
                }
            } else {
                errors.push(syn::Error::new(
                    path_ident.span(),
                    unknown_single_attr(path_ident, "std"),
                ));
            }
        }
    }

    let mut from_std = FromStd::No;
    let mut errors = Vec::new();

    attrs
        .iter()
        .for_each(|attr| inner(attr, &mut from_std, &mut errors));

    if let Some(e) = errors.into_iter().reduce(|mut a, b| {
        a.combine(b);
        a
    }) {
        return Err(e);
    }

    Ok(FieldAttr { from_std })
}

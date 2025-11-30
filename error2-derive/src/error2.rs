use std::str::FromStr;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use quote_use::quote_use;
use syn::{
    Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Token, Variant, Visibility, punctuated::Punctuated,
};

use crate::{
    messages::{
        AT_LEAST_ONE_FIELD, AT_LEAST_ONE_VARIANT, DISPLAY_SET_TWO_PLACE,
        DISPLAY_TOKENS_NOT_ON_ENUM, NO_DISPLAY_ON_ENUM_OR_VARIANT, NO_DISPLAY_ON_STRUCT,
        SUPPORTED_TYPES, incorrect_def,
    },
    parser::{parse_type_attr, parse_variant_attr},
    types::{
        ContextKind, ErrorKind, MyVariant, TypeAttr, TypeDisplayAttr, VariantAttr,
        VartiantDisplayAttr,
    },
};

fn crate_path() -> TokenStream {
    quote! { ::error2 }
}

pub(crate) fn generate(input: DeriveInput) -> syn::Result<TokenStream> {
    let crate_path = crate_path();

    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = input;

    let type_attr = parse_type_attr(&attrs)?;

    match data {
        Data::Union(DataUnion { union_token, .. }) => {
            Err(syn::Error::new(union_token.span, SUPPORTED_TYPES))
        }
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Unit => Err(syn::Error::new(ident.span(), SUPPORTED_TYPES)),
            Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => {
                Err(syn::Error::new(paren_token.span.join(), SUPPORTED_TYPES))
            }
            Fields::Named(FieldsNamed { brace_token, named }) => {
                if named.is_empty() {
                    Err(syn::Error::new(brace_token.span.join(), AT_LEAST_ONE_FIELD))
                } else {
                    generate_struct(&crate_path, type_attr, ident, generics, named)
                }
            }
        },
        syn::Data::Enum(DataEnum {
            brace_token,
            variants,
            ..
        }) => {
            if variants.is_empty() {
                return Err(syn::Error::new(
                    brace_token.span.join(),
                    AT_LEAST_ONE_VARIANT,
                ));
            }

            let mut errors = Vec::new();

            let variants = variants
                .into_iter()
                .filter_map(|vartiant| {
                    let Variant {
                        attrs,
                        ident,
                        fields,
                        ..
                    } = vartiant;

                    match fields {
                        Fields::Named(FieldsNamed { brace_token, named }) => {
                            if named.is_empty() {
                                errors.push(syn::Error::new(
                                    brace_token.span.join(),
                                    AT_LEAST_ONE_FIELD,
                                ));
                                None
                            } else {
                                Some(MyVariant {
                                    attrs,
                                    ident,
                                    named_fields: named,
                                })
                            }
                        }
                        Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => {
                            errors.push(syn::Error::new(paren_token.span.join(), SUPPORTED_TYPES));
                            None
                        }
                        Fields::Unit => {
                            errors.push(syn::Error::new(ident.span(), SUPPORTED_TYPES));
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();

            if let Some(e) = errors.into_iter().reduce(|mut a, b| {
                a.combine(b);
                a
            }) {
                return Err(e);
            }

            generate_enum(&crate_path, type_attr, ident, generics, variants)
        }
    }
}

fn generate_struct(
    crate_path: &TokenStream,
    type_attr: TypeAttr,
    struct_ident: Ident,
    generics: Generics,
    fields: Punctuated<Field, Token![,]>,
) -> syn::Result<TokenStream> {
    let TypeAttr {
        display: type_display,
        context_vis,
        mod_vis,
    } = type_attr;

    let display_tokens = match type_display {
        TypeDisplayAttr::None => {
            return Err(syn::Error::new(struct_ident.span(), NO_DISPLAY_ON_STRUCT));
        }
        TypeDisplayAttr::Disabled { .. } => None,
        TypeDisplayAttr::Enabled { tokens, .. } => Some(tokens),
    };

    let mut all_field_idents: Vec<&Ident> = Vec::with_capacity(fields.len());
    let mut no_source_no_backtrace_field_idents: Vec<&Ident> = Vec::with_capacity(fields.len());
    let mut no_source_no_backtrace_field_generics: Vec<Ident> = Vec::with_capacity(fields.len());
    let mut no_source_no_backtrace_generic_bounds: Vec<TokenStream> =
        Vec::with_capacity(fields.len());
    let mut source_field: Option<&Field> = None;
    let mut backtrace_field: Option<&Field> = None;

    for (i, field) in fields.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();

        all_field_idents.push(ident);

        if ident == "source" {
            source_field = Some(field);
        } else if ident == "backtrace" {
            backtrace_field = Some(field);
        } else {
            no_source_no_backtrace_field_idents.push(ident);
            no_source_no_backtrace_field_generics.push(format_ident!("__T{}", i));
            let ty = &field.ty;
            no_source_no_backtrace_generic_bounds.push(quote! { ::core::convert::Into<#ty> });
        }
    }

    let error_kind: ErrorKind;
    let source_type: TokenStream;
    let backtrace_field_tokens: TokenStream;
    let assert_source_not_impl_error2: TokenStream;

    match (source_field, backtrace_field) {
        // incorrect definition
        (None, None) => {
            return Err(syn::Error::new(
                struct_ident.span(),
                incorrect_def(ContextKind::Struct),
            ));
        }
        // root error
        (None, Some(_)) => {
            error_kind = ErrorKind::Root;
            source_type = quote! { #crate_path::NoneError };
            backtrace_field_tokens = quote! {
                backtrace: #crate_path::Backtrace::new(),
            };
            assert_source_not_impl_error2 = quote! {};
        }
        // error2 error
        (Some(source_field), None) => {
            error_kind = ErrorKind::Err2;
            source_type = {
                let ty = &source_field.ty;
                quote! { #ty }
            };
            backtrace_field_tokens = quote! {};
            assert_source_not_impl_error2 = quote! {};
        }
        // std error
        (Some(source_field), Some(_backtrace_field)) => {
            let ty = &source_field.ty;

            error_kind = ErrorKind::Std;
            source_type = quote! { #ty };
            backtrace_field_tokens = quote! {
                backtrace: #crate_path::Backtrace::with_head(
                    ::core::any::type_name_of_val(&source),
                    ::std::string::ToString::to_string(&source)
                ),
            };
            assert_source_not_impl_error2 = quote! {
                #crate_path::assert_not_impl_any!(#ty: #crate_path::Error2);
            }
        }
    }

    let display_body = match display_tokens {
        None => quote! {},
        Some(tokens) => quote! {
            #[allow(unused_variables)]
            #[allow(unused_assignments)]
            let Self { #(#all_field_idents,)* } = self;
            write!(f, #tokens)
        },
    };

    let error_source_body = if error_kind.is_root() {
        quote! {
            ::core::option::Option::None
        }
    } else {
        quote! {
            ::core::option::Option::Some(&self.source)
        }
    };

    let backtrace_body = match error_kind {
        ErrorKind::Root | ErrorKind::Std => quote! {
            &self.backtrace
        },
        ErrorKind::Err2 => quote! {
            #crate_path::Error2::backtrace(&self.source)
        },
    };

    let backtrace_mut_body = match error_kind {
        ErrorKind::Root | ErrorKind::Std => quote! {
            &mut self.backtrace
        },
        ErrorKind::Err2 => quote! {
            #crate_path::Error2::backtrace_mut(&mut self.source)
        },
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let context_def = generate_context_def(
        crate_path,
        &struct_ident,
        quote! { #struct_ident },
        &struct_ident,
        &context_vis,
        error_kind,
        no_source_no_backtrace_field_idents,
        no_source_no_backtrace_field_generics,
        no_source_no_backtrace_generic_bounds,
        &generics,
        source_type,
        backtrace_field_tokens,
        assert_source_not_impl_error2,
    );

    let expand = quote_use! {
        # use core::fmt::{Display, Formatter, Result};
        # use core::error::Error;
        # use core::option::Option;

        #context_def

        impl #impl_generics Display for #struct_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                #display_body
            }
        }

        impl #impl_generics Error for #struct_ident #ty_generics #where_clause {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                #error_source_body
            }
        }

        impl #impl_generics #crate_path::Error2 for #struct_ident #ty_generics #where_clause {
            #[inline]
            fn backtrace(&self) -> &#crate_path::Backtrace {
                #backtrace_body
            }

            #[inline]
            fn backtrace_mut(&mut self) -> &mut #crate_path::Backtrace {
                #backtrace_mut_body
            }
        }
    };

    let expand = if let Some(mod_vis) = mod_vis {
        let mod_name = struct_ident.to_string().to_snake_case();
        let mod_ident = Ident::new(&mod_name, struct_ident.span());

        quote! {
            #mod_vis mod #mod_ident {
                use super::*;

                #expand
            }
        }
    } else {
        expand
    };

    Ok(expand)
}

fn generate_enum(
    crate_path: &TokenStream,
    type_attr: TypeAttr,
    enum_ident: Ident,
    generics: Generics,
    variants: Vec<MyVariant>,
) -> syn::Result<TokenStream> {
    let TypeAttr {
        display: type_display,
        context_vis,
        mod_vis,
    } = type_attr;

    if let TypeDisplayAttr::Enabled { meta_span, .. } = type_display {
        return Err(syn::Error::new(meta_span, DISPLAY_TOKENS_NOT_ON_ENUM));
    }

    let mut context_defs = Vec::with_capacity(variants.len());
    let mut display_arms = Vec::with_capacity(variants.len());
    let mut error_source_arms = Vec::with_capacity(variants.len());
    let mut backtrace_arms = Vec::with_capacity(variants.len());
    let mut backtrace_mut_arms = Vec::with_capacity(variants.len());

    let mut errors = Vec::new();

    for variant in variants {
        let MyVariant {
            attrs: variant_attrs,
            ident: variant_ident,
            named_fields,
        } = variant;

        let VariantAttr {
            display: variant_display,
        } = match parse_variant_attr(&variant_attrs) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        let display_tokens = match (&type_display, variant_display) {
            (TypeDisplayAttr::Enabled { .. }, _) => unreachable!(),

            (TypeDisplayAttr::None, VartiantDisplayAttr::Enabled { tokens, .. }) => Some(tokens),
            (TypeDisplayAttr::Disabled { .. }, VartiantDisplayAttr::None) => None,

            (TypeDisplayAttr::None, VartiantDisplayAttr::None) => {
                errors.push(syn::Error::new(
                    enum_ident.span(),
                    NO_DISPLAY_ON_ENUM_OR_VARIANT,
                ));
                errors.push(syn::Error::new(
                    variant_ident.span(),
                    NO_DISPLAY_ON_ENUM_OR_VARIANT,
                ));
                continue;
            }

            (
                TypeDisplayAttr::Disabled {
                    meta_span: enum_meta_span,
                },
                VartiantDisplayAttr::Enabled {
                    meta_span: variant_meta_span,
                    ..
                },
            ) => {
                errors.push(syn::Error::new(*enum_meta_span, DISPLAY_SET_TWO_PLACE));
                errors.push(syn::Error::new(variant_meta_span, DISPLAY_SET_TWO_PLACE));
                continue;
            }
        };

        let mut all_field_idents: Vec<&Ident> = Vec::with_capacity(named_fields.len());
        let mut no_source_no_backtrace_field_idents: Vec<&Ident> =
            Vec::with_capacity(named_fields.len());
        let mut no_source_no_backtrace_field_generics: Vec<Ident> =
            Vec::with_capacity(named_fields.len());
        let mut no_source_no_backtrace_generic_bounds: Vec<TokenStream> =
            Vec::with_capacity(named_fields.len());
        let mut source_field: Option<&Field> = None;
        let mut backtrace_field: Option<&Field> = None;

        for (i, field) in named_fields.iter().enumerate() {
            let ident = field.ident.as_ref().unwrap();

            all_field_idents.push(ident);

            if ident == "source" {
                source_field = Some(field);
            } else if ident == "backtrace" {
                backtrace_field = Some(field);
            } else {
                no_source_no_backtrace_field_idents.push(ident);
                no_source_no_backtrace_field_generics.push(format_ident!("__T{}", i));
                let ty = &field.ty;
                no_source_no_backtrace_generic_bounds.push(quote! { ::core::convert::Into<#ty> });
            }
        }

        let error_kind: ErrorKind;
        let source_type: TokenStream;
        let backtrace_field_tokens: TokenStream;
        let assert_source_not_impl_error2: TokenStream;

        match (source_field, backtrace_field) {
            // incorrect definition
            (None, None) => {
                errors.push(syn::Error::new(
                    variant_ident.span(),
                    incorrect_def(ContextKind::Variant),
                ));
                continue;
            }
            // root error
            (None, Some(_)) => {
                error_kind = ErrorKind::Root;
                source_type = quote! { #crate_path::NoneError };
                backtrace_field_tokens = quote! {
                    backtrace: #crate_path::Backtrace::new(),
                };
                assert_source_not_impl_error2 = quote! {};
            }
            // error2 error
            (Some(source_field), None) => {
                error_kind = ErrorKind::Err2;
                source_type = {
                    let ty = &source_field.ty;
                    quote! { #ty }
                };
                backtrace_field_tokens = quote! {};
                assert_source_not_impl_error2 = quote! {};
            }
            // std error
            (Some(source_field), Some(_backtrace_field)) => {
                let ty = &source_field.ty;

                error_kind = ErrorKind::Std;
                source_type = quote! { #ty };
                backtrace_field_tokens = quote! {
                    backtrace: #crate_path::Backtrace::with_head(
                        ::core::any::type_name_of_val(&source),
                        ::std::string::ToString::to_string(&source)
                    ),
                };
                assert_source_not_impl_error2 = quote! {
                    #crate_path::assert_not_impl_any!(#ty: #crate_path::Error2);
                }
            }
        }

        let VariantTokens {
            context_def,
            display_arm,
            error_source_arm,
            backtrace_arm,
            backtrace_mut_arm,
        } = generate_variant(
            crate_path,
            &enum_ident,
            &variant_ident,
            &context_vis,
            error_kind,
            all_field_idents,
            no_source_no_backtrace_field_idents,
            no_source_no_backtrace_field_generics,
            no_source_no_backtrace_generic_bounds,
            &generics,
            source_type,
            backtrace_field_tokens,
            assert_source_not_impl_error2,
            display_tokens,
        );

        context_defs.push(context_def);
        display_arms.push(display_arm);
        error_source_arms.push(error_source_arm);
        backtrace_arms.push(backtrace_arm);
        backtrace_mut_arms.push(backtrace_mut_arm);
    }

    if let Some(e) = errors.into_iter().reduce(|mut a, b| {
        a.combine(b);
        a
    }) {
        return Err(e);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expand = quote_use! {
        # use core::fmt::{Display, Formatter, Result};
        # use core::error::Error;
        # use core::option::Option;

        #(#context_defs)*

        impl #impl_generics Display for #enum_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                #[allow(unused_variables)]
                #[allow(unused_assignments)]
                match self {
                    #(#display_arms)*
                }
            }
        }

        impl #impl_generics Error for #enum_ident #ty_generics #where_clause {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match self {
                    #(#error_source_arms)*
                }
            }
        }

        impl #impl_generics #crate_path::Error2 for #enum_ident #ty_generics #where_clause {
            #[inline]
            fn backtrace(&self) -> &#crate_path::Backtrace {
                match self {
                    #(#backtrace_arms)*
                }
            }

            #[inline]
            fn backtrace_mut(&mut self) -> &mut #crate_path::Backtrace {
                match self {
                    #(#backtrace_mut_arms)*
                }
            }
        }
    };

    let expand = if let Some(mod_vis) = mod_vis {
        let mod_name = enum_ident.to_string().to_snake_case();
        let mod_ident = Ident::new(&mod_name, enum_ident.span());

        quote! {
            #mod_vis mod #mod_ident {
                use super::*;

                #expand
            }
        }
    } else {
        expand
    };

    Ok(expand)
}

#[expect(clippy::too_many_arguments)]
fn generate_variant(
    crate_path: &TokenStream,
    enum_ident: &Ident,
    variant_ident: &Ident,
    vis: &Visibility,
    error_kind: ErrorKind,
    all_field_idents: Vec<&Ident>,
    no_source_no_backtrace_field_idents: Vec<&Ident>,
    no_source_no_backtrace_field_generics: Vec<Ident>,
    no_source_no_backtrace_generic_bounds: Vec<TokenStream>,
    generics: &Generics,
    source_type: TokenStream,
    backtrace_field_tokens: TokenStream,
    assert_source_not_impl_error2: TokenStream,
    display_tokens: Option<TokenStream>,
) -> VariantTokens {
    let context_def = generate_context_def(
        crate_path,
        enum_ident,
        quote! { #enum_ident::#variant_ident },
        variant_ident,
        vis,
        error_kind,
        no_source_no_backtrace_field_idents,
        no_source_no_backtrace_field_generics,
        no_source_no_backtrace_generic_bounds,
        generics,
        source_type,
        backtrace_field_tokens,
        assert_source_not_impl_error2,
    );

    let display_arm = match display_tokens {
        None => quote! {},
        Some(tokens) => quote! {
            Self::#variant_ident { #(#all_field_idents,)* } => {
                write!(f, #tokens)
            }
        },
    };

    let error_source_arm = if error_kind.is_root() {
        quote! {
            Self::#variant_ident { .. } => ::core::option::Option::None,
        }
    } else {
        quote! {
            Self::#variant_ident { source, .. } => ::core::option::Option::Some(source),
        }
    };

    let backtrace_arm = match error_kind {
        ErrorKind::Root | ErrorKind::Std => quote! {
            Self::#variant_ident { backtrace, .. } => backtrace,
        },
        ErrorKind::Err2 => quote! {
            Self::#variant_ident { source, .. } => #crate_path::Error2::backtrace(source),
        },
    };

    let backtrace_mut_arm = match error_kind {
        ErrorKind::Root | ErrorKind::Std => quote! {
            Self::#variant_ident { backtrace, .. } => backtrace,
        },
        ErrorKind::Err2 => quote! {
            Self::#variant_ident { source, .. } => #crate_path::Error2::backtrace_mut(source),
        },
    };

    VariantTokens {
        context_def,
        display_arm,
        error_source_arm,
        backtrace_arm,
        backtrace_mut_arm,
    }
}

struct VariantTokens {
    context_def: TokenStream,
    display_arm: TokenStream,
    error_source_arm: TokenStream,
    backtrace_arm: TokenStream,
    backtrace_mut_arm: TokenStream,
}

#[expect(clippy::too_many_arguments)]
fn generate_context_def(
    crate_path: &TokenStream,
    type_ident: &Ident,
    type_path: TokenStream,
    context_ident_prefix: &Ident,
    context_vis: &Visibility,
    error_kind: ErrorKind,
    no_source_no_backtrace_field_idents: Vec<&Ident>,
    no_source_no_backtrace_field_generics: Vec<Ident>,
    no_source_no_backtrace_generic_bounds: Vec<TokenStream>,
    generics: &Generics,
    source_type: TokenStream,
    backtrace_field_tokens: TokenStream,
    assert_source_not_impl_error2: TokenStream,
) -> TokenStream {
    let context_ident = format_ident!("{}2", context_ident_prefix);

    let context_struct_body = if no_source_no_backtrace_field_idents.is_empty() {
        quote! { ; }
    } else {
        quote! {
            {
                #(
                    #[allow(missing_docs)]
                    #context_vis #no_source_no_backtrace_field_idents : #no_source_no_backtrace_field_generics,
                )*
            }
        }
    };

    let context_generics = if no_source_no_backtrace_field_idents.is_empty() {
        quote! {}
    } else {
        quote! { < #(#no_source_no_backtrace_field_generics,)* > }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let additional_impl_generics = if no_source_no_backtrace_field_idents.is_empty() {
        quote! { #impl_generics }
    } else {
        let mut impl_generics = quote! { #impl_generics }.to_string();

        match impl_generics.find('<') {
            Some(pos) => {
                let mut buf = String::new();

                for g in &no_source_no_backtrace_field_generics {
                    buf.push_str(&g.to_string());
                    buf.push_str(", ");
                }

                impl_generics.insert_str(pos + 1, &buf);
                TokenStream::from_str(&impl_generics).unwrap()
            }
            _ => quote! { < #(#no_source_no_backtrace_field_generics,)* > },
        }
    };

    let fail_methods_impl_generics = {
        let mut impl_generics = quote! { #impl_generics }.to_string();

        match impl_generics.find('<') {
            Some(pos) => {
                impl_generics.insert_str(pos + 1, "__T, ");
                TokenStream::from_str(&impl_generics).unwrap()
            }
            _ => quote! { <__T> },
        }
    };

    let additional_where_clause = if no_source_no_backtrace_field_idents.is_empty() {
        quote! { #where_clause }
    } else {
        match where_clause {
            Some(where_clause) => {
                let where_clauses = where_clause.predicates.iter().collect::<Vec<_>>();

                quote! {
                    where
                    #(
                        #where_clauses,
                    )*
                    #(
                        #no_source_no_backtrace_field_generics : #no_source_no_backtrace_generic_bounds,
                    )*
                }
            }
            None => {
                quote! {
                    where
                    #(
                        #no_source_no_backtrace_field_generics : #no_source_no_backtrace_generic_bounds,
                    )*
                }
            }
        }
    };

    let root_error_methods = if !error_kind.is_root() {
        quote! {}
    } else {
        quote_use! {
            # use core::convert::Into;
            # use core::result::Result;
            # use core::any::type_name_of_val;
            # use std::string::ToString;
            # use #crate_path::{ErrorWrap, NoneError, Location};

            impl #context_generics #context_ident #context_generics {
                #[inline]
                #[must_use]
                #[track_caller]
                #[allow(dead_code)]
                #context_vis fn build #impl_generics (self) -> #type_ident #ty_generics #additional_where_clause {
                    Self::build_with_location(self, Location::caller())
                }

                #[inline]
                #[must_use]
                #context_vis fn build_with_location #impl_generics (self, location: Location) -> #type_ident #ty_generics #additional_where_clause {
                    <Self as ErrorWrap < NoneError, #type_ident #ty_generics > >::wrap(self, NoneError, location)
                }

                #[inline]
                #[track_caller]
                #[allow(dead_code)]
                #context_vis fn fail #fail_methods_impl_generics (self) -> Result<__T, #type_ident #ty_generics> #additional_where_clause {
                    Self::fail_with_location(self, Location::caller())
                }

                #[inline]
                #[allow(dead_code)]
                #context_vis fn fail_with_location #fail_methods_impl_generics (self, location: Location) -> Result<__T, #type_ident #ty_generics> #additional_where_clause {
                    Result::Err(self.build_with_location(location))
                }
            }
        }
    };

    let source_field = if error_kind.is_root() {
        quote! {}
    } else {
        quote! {
            source,
        }
    };

    quote_use! {
        # use core::convert::Into;
        # use core::any::type_name_of_val;
        # use std::string::ToString;
        # use #crate_path::{ErrorWrap, Error2, Location};

        #[derive(Debug, Copy, Clone)]
        #context_vis struct #context_ident #context_generics #context_struct_body

        #root_error_methods

        impl #additional_impl_generics ErrorWrap < #source_type, #type_ident #ty_generics > for #context_ident #context_generics #additional_where_clause {
            #[allow(unused_variables)]
            fn wrap(self, source: #source_type, location: Location) -> #type_ident #ty_generics {
                #assert_source_not_impl_error2

                let mut error = #type_path {
                    #(
                        #no_source_no_backtrace_field_idents : Into::into(self.#no_source_no_backtrace_field_idents),
                    )*
                    #backtrace_field_tokens
                    #source_field
                };

                let type_name = type_name_of_val(&error);
                let display = ToString::to_string(&error);

                Error2::backtrace_mut(&mut error).push_error(type_name, display, location);

                error
            }
        }
    }
}

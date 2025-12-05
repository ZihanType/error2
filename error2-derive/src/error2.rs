use std::str::FromStr;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use quote_use::quote_use;
use syn::{
    Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Path, Token, Type, Variant, Visibility, parse_quote, punctuated::Punctuated,
};

use crate::{
    generics::{InferredBounds, ParamsInScope},
    messages::{
        AT_LEAST_ONE_FIELD, AT_LEAST_ONE_VARIANT, DISPLAY_TOKENS_NOT_ON_ENUM,
        MISSING_DISPLAY_ON_VARIANT, SUPPORTED_TYPES, incorrect_def,
    },
    parser::{parse_type_attr, parse_variant_attr},
    types::{ContextKind, ErrorKind, MyVariant, Trait, TypeAttr, TypeDisplayAttr, VariantAttr},
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

    let scope = ParamsInScope::new(&generics);

    let mut error_inferred_bounds = InferredBounds::with_capacity(3);
    if generics.type_params().next().is_some() {
        let (_, ty_generics, _) = generics.split_for_impl();
        let ty = quote! {
            #ident #ty_generics
        };
        error_inferred_bounds.insert(ty.clone(), Trait::Debug);
        error_inferred_bounds.insert(ty, Trait::Display);
    }

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
                    generate_struct(
                        &crate_path,
                        type_attr,
                        ident,
                        &generics,
                        named,
                        &scope,
                        error_inferred_bounds,
                    )
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

            generate_enum(
                &crate_path,
                type_attr,
                ident,
                &generics,
                variants,
                &scope,
                error_inferred_bounds,
            )
        }
    }
}

fn generate_struct(
    crate_path: &TokenStream,
    type_attr: TypeAttr,
    struct_ident: Ident,
    generics: &Generics,
    fields: Punctuated<Field, Token![,]>,
    scope: &ParamsInScope,
    mut error_inferred_bounds: InferredBounds,
) -> syn::Result<TokenStream> {
    let TypeAttr {
        display: type_display,
        context_vis,
        mod_vis,
    } = type_attr;

    let display_tokens = match type_display {
        TypeDisplayAttr::None => None,
        TypeDisplayAttr::Enabled { tokens, .. } => Some(tokens),
    };

    let mut all_field_idents: Vec<&Ident> = Vec::with_capacity(fields.len());
    let mut no_source_no_backtrace_field_idents: Vec<&Ident> = Vec::with_capacity(fields.len());
    let mut no_source_no_backtrace_field_generics: Vec<Ident> = Vec::with_capacity(fields.len());
    let mut no_source_no_backtrace_inferred_bounds = InferredBounds::with_capacity(fields.len());
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
            let generic = format_ident!("__T{}", i);
            no_source_no_backtrace_field_generics.push(generic.clone());
            let ty = &field.ty;
            no_source_no_backtrace_inferred_bounds
                .insert(generic, quote! { ::core::convert::Into<#ty> });
        }
    }

    let error_kind: ErrorKind;
    let source_type: Type;
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
            source_type = parse_quote! { #crate_path::NoneError };
            backtrace_field_tokens = quote! {
                backtrace: #crate_path::Backtrace::new(),
            };
            assert_source_not_impl_error2 = quote! {};
        }
        // error2 error
        (Some(source_field), None) => {
            let ty = &source_field.ty;

            error_kind = ErrorKind::Err2;
            source_type = ty.clone();
            backtrace_field_tokens = quote! {};
            assert_source_not_impl_error2 = quote! {};
            if scope.intersects(ty) {
                error_inferred_bounds.insert(ty, quote! { #crate_path::Error2 + 'static });
            }
        }
        // std error
        (Some(source_field), Some(_backtrace_field)) => {
            let ty = &source_field.ty;

            error_kind = ErrorKind::Std;
            source_type = ty.clone();
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

    let context_def = generate_context_def(
        crate_path,
        &struct_ident,
        parse_quote! { #struct_ident },
        &struct_ident,
        &context_vis,
        error_kind,
        no_source_no_backtrace_field_idents,
        no_source_no_backtrace_field_generics,
        no_source_no_backtrace_inferred_bounds.merge(&error_inferred_bounds),
        generics,
        source_type,
        backtrace_field_tokens,
        assert_source_not_impl_error2,
    );

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let display_impl = match display_tokens {
        None => quote! {},
        Some(tokens) => quote_use! {
            # use core::fmt::{Display, Formatter, Result};

            impl #impl_generics Display for #struct_ident #ty_generics #where_clause {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    #[allow(unused_variables)]
                    #[allow(unused_assignments)]
                    let Self { #(#all_field_idents,)* } = self;
                    write!(f, #tokens)
                }
            }
        },
    };

    let error_where_clause = error_inferred_bounds.augment_where_clause(where_clause.cloned());

    let expand = quote_use! {
        # use core::error::Error;
        # use core::option::Option;

        #context_def

        #display_impl

        impl #impl_generics Error for #struct_ident #ty_generics #error_where_clause {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                #error_source_body
            }
        }

        impl #impl_generics #crate_path::Error2 for #struct_ident #ty_generics #error_where_clause {
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
    generics: &Generics,
    variants: Vec<MyVariant>,
    scope: &ParamsInScope,
    mut error_inferred_bounds: InferredBounds,
) -> syn::Result<TokenStream> {
    let TypeAttr {
        display: type_display,
        context_vis,
        mod_vis,
    } = type_attr;

    if let TypeDisplayAttr::Enabled { meta_span, .. } = type_display {
        return Err(syn::Error::new(meta_span, DISPLAY_TOKENS_NOT_ON_ENUM));
    };

    let mut errors = Vec::new();

    let mut inputs: Vec<VariantInput> = Vec::with_capacity(variants.len());
    let mut exist_display_on_variant = false;

    for variant in &variants {
        let MyVariant {
            attrs: variant_attrs,
            ident: variant_ident,
            named_fields,
        } = variant;

        let VariantAttr {
            display: variant_display,
        } = match parse_variant_attr(variant_attrs) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        if variant_display.is_some() {
            exist_display_on_variant = true;
        }

        let mut all_field_idents: Vec<&Ident> = Vec::with_capacity(named_fields.len());
        let mut no_source_no_backtrace_field_idents: Vec<&Ident> =
            Vec::with_capacity(named_fields.len());
        let mut no_source_no_backtrace_field_generics: Vec<Ident> =
            Vec::with_capacity(named_fields.len());
        let mut no_source_no_backtrace_inferred_bounds =
            InferredBounds::with_capacity(named_fields.len());
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
                let generic = format_ident!("__T{}", i);
                no_source_no_backtrace_field_generics.push(generic.clone());
                let ty = &field.ty;
                no_source_no_backtrace_inferred_bounds
                    .insert(generic, quote! { ::core::convert::Into<#ty> });
            }
        }

        let error_kind: ErrorKind;
        let source_type: Type;
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
                source_type = parse_quote! { #crate_path::NoneError };
                backtrace_field_tokens = quote! {
                    backtrace: #crate_path::Backtrace::new(),
                };
                assert_source_not_impl_error2 = quote! {};
            }
            // error2 error
            (Some(source_field), None) => {
                let ty = &source_field.ty;

                error_kind = ErrorKind::Err2;
                source_type = ty.clone();
                backtrace_field_tokens = quote! {};
                assert_source_not_impl_error2 = quote! {};
                if scope.intersects(ty) {
                    error_inferred_bounds.insert(ty, quote! { #crate_path::Error2 + 'static });
                }
            }
            // std error
            (Some(source_field), Some(_backtrace_field)) => {
                let ty = &source_field.ty;

                error_kind = ErrorKind::Std;
                source_type = ty.clone();
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

        inputs.push(VariantInput {
            variant_ident,
            error_kind,
            all_field_idents,
            no_source_no_backtrace_field_idents,
            no_source_no_backtrace_field_generics,
            no_source_no_backtrace_inferred_bounds,
            source_type,
            backtrace_field_tokens,
            assert_source_not_impl_error2,
            variant_display,
        });
    }

    let mut context_defs = Vec::with_capacity(variants.len());
    let mut display_arms = Vec::with_capacity(variants.len());
    let mut error_source_arms = Vec::with_capacity(variants.len());
    let mut backtrace_arms = Vec::with_capacity(variants.len());
    let mut backtrace_mut_arms = Vec::with_capacity(variants.len());

    for input in inputs {
        let VariantInput {
            variant_ident,
            error_kind,
            all_field_idents,
            no_source_no_backtrace_field_idents,
            no_source_no_backtrace_field_generics,
            no_source_no_backtrace_inferred_bounds,
            source_type,
            backtrace_field_tokens,
            assert_source_not_impl_error2,
            variant_display,
        } = input;

        if variant_display.is_none() && exist_display_on_variant {
            errors.push(syn::Error::new(
                variant_ident.span(),
                MISSING_DISPLAY_ON_VARIANT,
            ));
            continue;
        }

        let VariantOutput {
            context_def,
            display_arm,
            error_source_arm,
            backtrace_arm,
            backtrace_mut_arm,
        } = generate_variant(
            crate_path,
            &enum_ident,
            variant_ident,
            &context_vis,
            error_kind,
            all_field_idents,
            no_source_no_backtrace_field_idents,
            no_source_no_backtrace_field_generics,
            no_source_no_backtrace_inferred_bounds.merge(&error_inferred_bounds),
            generics,
            source_type,
            backtrace_field_tokens,
            assert_source_not_impl_error2,
            variant_display,
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

    let display_impl = if !exist_display_on_variant {
        quote! {}
    } else {
        quote_use! {
            # use core::fmt::{Display, Formatter, Result};

            impl #impl_generics Display for #enum_ident #ty_generics #where_clause {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    #[allow(unused_variables)]
                    #[allow(unused_assignments)]
                    match self {
                        #(#display_arms)*
                    }
                }
            }
        }
    };

    let error_where_clause = error_inferred_bounds.augment_where_clause(where_clause.cloned());

    let expand = quote_use! {
        # use core::error::Error;
        # use core::option::Option;

        #(#context_defs)*

        #display_impl

        impl #impl_generics Error for #enum_ident #ty_generics #error_where_clause {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match self {
                    #(#error_source_arms)*
                }
            }
        }

        impl #impl_generics #crate_path::Error2 for #enum_ident #ty_generics #error_where_clause {
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
    all_inferred_bounds: InferredBounds,
    generics: &Generics,
    source_type: Type,
    backtrace_field_tokens: TokenStream,
    assert_source_not_impl_error2: TokenStream,
    display_tokens: Option<TokenStream>,
) -> VariantOutput {
    let context_def = generate_context_def(
        crate_path,
        enum_ident,
        parse_quote! { #enum_ident::#variant_ident },
        variant_ident,
        vis,
        error_kind,
        no_source_no_backtrace_field_idents,
        no_source_no_backtrace_field_generics,
        all_inferred_bounds,
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

    VariantOutput {
        context_def,
        display_arm,
        error_source_arm,
        backtrace_arm,
        backtrace_mut_arm,
    }
}

struct VariantInput<'a> {
    variant_ident: &'a Ident,
    error_kind: ErrorKind,
    all_field_idents: Vec<&'a Ident>,
    no_source_no_backtrace_field_idents: Vec<&'a Ident>,
    no_source_no_backtrace_field_generics: Vec<Ident>,
    no_source_no_backtrace_inferred_bounds: InferredBounds,
    source_type: Type,
    backtrace_field_tokens: TokenStream,
    assert_source_not_impl_error2: TokenStream,
    variant_display: Option<TokenStream>,
}

struct VariantOutput {
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
    type_path: Path,
    context_ident_prefix: &Ident,
    context_vis: &Visibility,
    error_kind: ErrorKind,
    no_source_no_backtrace_field_idents: Vec<&Ident>,
    no_source_no_backtrace_field_generics: Vec<Ident>,
    all_inferred_bounds: InferredBounds,
    generics: &Generics,
    source_type: Type,
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
        let mut impl_generics = impl_generics.to_token_stream().to_string();

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
        let mut impl_generics = impl_generics.to_token_stream().to_string();

        match impl_generics.find('<') {
            Some(pos) => {
                impl_generics.insert_str(pos + 1, "__T, ");
                TokenStream::from_str(&impl_generics).unwrap()
            }
            _ => quote! { <__T> },
        }
    };

    let where_clause = all_inferred_bounds.augment_where_clause(where_clause.cloned());

    let root_error_methods = if !error_kind.is_root() {
        quote! {}
    } else {
        quote_use! {
            # use core::result::Result;
            # use #crate_path::{ErrorWrap, NoneError, Location};

            impl #context_generics #context_ident #context_generics {
                #[inline]
                #[must_use]
                #[track_caller]
                #[allow(dead_code)]
                #context_vis fn build #impl_generics (self) -> #type_ident #ty_generics #where_clause {
                    Self::build_with_location(self, Location::caller())
                }

                #[inline]
                #[must_use]
                #context_vis fn build_with_location #impl_generics (self, location: Location) -> #type_ident #ty_generics #where_clause {
                    <Self as ErrorWrap < NoneError, #type_ident #ty_generics > >::wrap(self, NoneError, location)
                }

                #[inline]
                #[track_caller]
                #[allow(dead_code)]
                #context_vis fn fail #fail_methods_impl_generics (self) -> Result<__T, #type_ident #ty_generics> #where_clause {
                    Self::fail_with_location(self, Location::caller())
                }

                #[inline]
                #[allow(dead_code)]
                #context_vis fn fail_with_location #fail_methods_impl_generics (self, location: Location) -> Result<__T, #type_ident #ty_generics> #where_clause {
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

        #[derive(Debug, Clone, Copy)]
        #context_vis struct #context_ident #context_generics #context_struct_body

        #root_error_methods

        impl #additional_impl_generics ErrorWrap < #source_type, #type_ident #ty_generics > for #context_ident #context_generics #where_clause {
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

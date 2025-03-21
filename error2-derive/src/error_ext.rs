use from_attr::{AttrsValue, FromAttr};
use proc_macro2::TokenStream;
use quote_use::quote_use;
use syn::{
    DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Path, Token, Variant, punctuated::Punctuated,
};

#[derive(FromAttr)]
#[attribute(idents = [error2])]
struct FieldAttr {
    #[attribute(rename = "std")]
    from_std: bool,
    convert: Option<Path>,
}

struct MyVariant {
    ident: Ident,
    fields: Punctuated<Field, Token![,]>,
}

const MSG: &str = "`ErrorExt` can only be derived for structs and enums with named fields";

pub(crate) fn generate(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    match data {
        syn::Data::Struct(DataStruct {
            struct_token,
            fields,
            ..
        }) => match fields {
            Fields::Named(FieldsNamed { brace_token, named }) => {
                if named.is_empty() {
                    Err(syn::Error::new(
                        brace_token.span.join(),
                        "must have at least one field",
                    ))
                } else {
                    generate_struct(ident, generics, named)
                }
            }
            Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => {
                Err(syn::Error::new(paren_token.span.join(), MSG))
            }
            Fields::Unit => Err(syn::Error::new(struct_token.span, MSG)),
        },
        syn::Data::Enum(DataEnum {
            brace_token,
            variants,
            ..
        }) => {
            if variants.is_empty() {
                return Err(syn::Error::new(
                    brace_token.span.join(),
                    "must have at least one variant",
                ));
            }

            let mut errors = Vec::new();

            let variants = variants
                .into_iter()
                .filter_map(|vartiant| {
                    let Variant { ident, fields, .. } = vartiant;

                    match fields {
                        Fields::Named(FieldsNamed { brace_token, named }) => {
                            if named.is_empty() {
                                errors.push(syn::Error::new(
                                    brace_token.span.join(),
                                    "must have at least one field",
                                ));
                                None
                            } else {
                                Some(MyVariant {
                                    ident,
                                    fields: named,
                                })
                            }
                        }
                        Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => {
                            errors.push(syn::Error::new(paren_token.span.join(), MSG));
                            None
                        }
                        Fields::Unit => {
                            errors.push(syn::Error::new(ident.span(), MSG));
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

            generate_enum(ident, generics, variants)
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            Err(syn::Error::new(union_token.span, MSG))
        }
    }
}

fn generate_struct(
    ident: Ident,
    generics: Generics,
    fields: Punctuated<Field, Token![,]>,
) -> syn::Result<TokenStream> {
    let source = fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "source");

    let entry_body = match source {
        None => quote_use! {
            # use error2::NextError;

            (&self.locations, NextError::None)
        },
        Some(source) => match FieldAttr::from_attributes(&source.attrs) {
            Ok(Some(AttrsValue {
                value: FieldAttr { from_std, convert },
                ..
            })) => match (from_std, convert) {
                (true, None) => quote_use! {
                    # use error2::NextError;

                    (&self.locations, NextError::Std(&self.source))
                },
                (true, Some(convert)) => quote_use! {
                    # use error2::NextError;

                    (&self.locations, NextError::Std(#convert(&self.source)))
                },
                (false, None) => quote_use! {
                    # use error2::NextError;

                    (&self.locations, NextError::Ext(&self.source))
                },
                (false, Some(convert)) => quote_use! {
                    # use error2::NextError;

                    (&self.locations, NextError::Ext(#convert(&self.source)))
                },
            },
            Ok(None) => quote_use! {
                # use error2::NextError;

                (&self.locations, NextError::Ext(&self.source))
            },
            Err(AttrsValue { value: e, .. }) => return Err(e),
        },
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expand = quote_use! {
        # use error2::{ErrorExt, Locations, NextError};

        impl #impl_generics ErrorExt for #ident #ty_generics #where_clause {
            fn entry(&self) -> (&Locations, NextError<'_>) {
                #entry_body
            }

            fn locations(&mut self) -> &mut Locations {
                &mut self.locations
            }
        }
    };

    Ok(expand)
}

fn generate_enum(
    ident: Ident,
    generics: Generics,
    variants: Vec<MyVariant>,
) -> syn::Result<TokenStream> {
    let mut entry_arms = Vec::with_capacity(variants.len());
    let mut locations_arms = Vec::with_capacity(variants.len());
    let mut errors = Vec::new();

    for variant in variants {
        let MyVariant {
            ident: variant_ident,
            fields,
        } = variant;

        let source = fields
            .iter()
            .find(|f| f.ident.as_ref().unwrap() == "source");

        let entry_arm = match source {
            None => quote_use! {
                # use error2::NextError;

                Self::#variant_ident { locations, .. } => (locations, NextError::None),
            },
            Some(source) => match FieldAttr::from_attributes(&source.attrs) {
                Ok(Some(AttrsValue {
                    value: FieldAttr { from_std, convert },
                    ..
                })) => match (from_std, convert) {
                    (true, None) => quote_use! {
                        # use error2::NextError;

                        Self::#variant_ident { locations, source, .. } => (locations, NextError::Std(source)),
                    },
                    (true, Some(convert)) => quote_use! {
                        # use error2::NextError;

                        Self::#variant_ident { locations, source, .. } => (locations, NextError::Std(#convert(source))),
                    },
                    (false, None) => quote_use! {
                        # use error2::NextError;

                        Self::#variant_ident { locations, source, .. } => (locations, NextError::Ext(source)),
                    },
                    (false, Some(convert)) => quote_use! {
                        # use error2::NextError;

                        Self::#variant_ident { locations, source, .. } => (locations, NextError::Ext(#convert(source))),
                    },
                },
                Ok(None) => quote_use! {
                    # use error2::NextError;

                    Self::#variant_ident { locations, source, .. } => (locations, NextError::Ext(source)),
                },
                Err(AttrsValue { value: e, .. }) => {
                    errors.push(e);
                    continue;
                }
            },
        };

        let locations_arm = quote_use! {
            Self::#variant_ident { locations, .. } => locations,
        };

        entry_arms.push(entry_arm);
        locations_arms.push(locations_arm);
    }

    if let Some(e) = errors.into_iter().reduce(|mut a, b| {
        a.combine(b);
        a
    }) {
        return Err(e);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expand = quote_use! {
        # use error2::{ErrorExt, Locations, NextError};

        impl #impl_generics ErrorExt for #ident #ty_generics #where_clause {
            fn entry(&self) -> (&Locations, NextError<'_>) {
                match self {
                    #(#entry_arms)*
                }
            }

            fn locations(&mut self) -> &mut Locations {
                match self {
                    #(#locations_arms)*
                }
            }
        }
    };

    Ok(expand)
}

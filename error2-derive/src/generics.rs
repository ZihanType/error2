use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    GenericArgument, Generics, Ident, PathArguments, Token, Type, WhereClause, parse_quote,
    punctuated::Punctuated,
};

pub(crate) struct ParamsInScope<'a> {
    names: HashSet<&'a Ident>,
}

impl<'a> ParamsInScope<'a> {
    pub(crate) fn new(generics: &'a Generics) -> Self {
        ParamsInScope {
            names: generics.type_params().map(|param| &param.ident).collect(),
        }
    }

    pub(crate) fn intersects(&self, ty: &Type) -> bool {
        let mut found = false;
        crawl(self, ty, &mut found);
        found
    }
}

fn crawl(in_scope: &ParamsInScope, ty: &Type, found: &mut bool) {
    let Type::Path(ty) = ty else { return };

    if let Some(qself) = &ty.qself {
        crawl(in_scope, &qself.ty, found);
    } else {
        let front = ty.path.segments.first().unwrap();
        if front.arguments.is_none() && in_scope.names.contains(&front.ident) {
            *found = true;
        }
    }

    for segment in &ty.path.segments {
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            continue;
        };

        for arg in &arguments.args {
            if let GenericArgument::Type(ty) = arg {
                crawl(in_scope, ty, found);
            }
        }
    }
}

// IndexMap<Type, IndexSet<TypeParamBound>>
pub(crate) struct InferredBounds {
    bounds: HashMap<String, (HashSet<String>, Punctuated<TokenStream, Token![+]>)>,
    order: Vec<TokenStream>,
}

impl InferredBounds {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        InferredBounds {
            bounds: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn insert(&mut self, ty: impl ToTokens, bound: impl ToTokens) {
        let ty = ty.to_token_stream();
        let bound = bound.to_token_stream();

        let (set, tokens) = self.bounds.entry(ty.to_string()).or_default();
        debug_assert_eq!(set.len(), tokens.len());

        if set.is_empty() {
            self.order.push(ty);
        }

        if set.insert(bound.to_string()) {
            tokens.push(bound);
        }
    }

    pub(crate) fn merge(mut self, other: &Self) -> Self {
        for ty in &other.order {
            let ty_str = ty.to_string();

            let (_, other_tokens) = other.bounds.get(&ty_str).unwrap();

            let (set, tokens) = self.bounds.entry(ty_str).or_default();

            if set.is_empty() {
                self.order.push(ty.clone());
            }

            for other_bound in other_tokens {
                if set.insert(other_bound.to_string()) {
                    tokens.push(other_bound.clone());
                }
            }
        }

        self
    }

    pub(crate) fn augment_where_clause(
        &self,
        where_clause: Option<WhereClause>,
    ) -> Option<WhereClause> {
        if self.order.is_empty() {
            return where_clause;
        }

        let mut where_clause = where_clause.unwrap_or_else(|| WhereClause {
            where_token: <Token![where]>::default(),
            predicates: Punctuated::new(),
        });

        for ty in &self.order {
            let (_, bounds) = &self.bounds.get(&ty.to_string()).unwrap();
            where_clause.predicates.push(parse_quote!(#ty: #bounds));
        }

        Some(where_clause)
    }
}

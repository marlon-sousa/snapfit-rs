//! Adapter: finds where a signature or a bound names `Self` itself, and what
//! a `where` clause bounds `Self` by. Plays the role of its parent.

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit::{self, Visit};

pub(super) use crate::adapters::syntax::{is_bare_self, unwrapped};
use crate::domain::{SelfBound, SelfUse};

/// `Self` where an implementation's type could not stand for it: anywhere
/// but a `Self::` path and the bounded type of a bound on `Self`. The
/// receiver and a whole `-> Self` are left to the caller, which never visits them.
pub(super) struct SelfFinder(pub(super) SelfUse);

impl SelfFinder {
    pub(super) fn new() -> Self {
        Self(SelfUse::Forwardable)
    }
}

impl<'ast> Visit<'ast> for SelfFinder {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        match &path.qself {
            None if path.path.is_ident("Self") => self.0 = SelfUse::Elsewhere,
            Some(qself) if is_bare_self(unwrapped(&qself.ty)) => self.visit_path(&path.path),
            _ => visit::visit_type_path(self, path),
        }
    }

    fn visit_predicate_type(&mut self, predicate: &'ast syn::PredicateType) {
        if is_bare_self(unwrapped(&predicate.bounded_ty)) {
            for bound in &predicate.bounds {
                self.visit_type_param_bound(bound);
            }
        } else {
            visit::visit_predicate_type(self, predicate);
        }
    }
}

/// Whether `tokens` name `Self` anywhere, also inside the group a
/// `macro_rules!` puts around what it passes.
pub(super) fn names_self(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => ident == "Self",
        TokenTree::Group(group) => names_self(group.stream()),
        _ => false,
    })
}

/// Whether the trait's parameters or `where` clause name `Self`, which the
/// impls for the dispatch types could only hold through themselves, other
/// than as the bounded type of a bound on `Self` that names it nowhere else.
pub(super) fn generics_name_self(generics: &syn::Generics) -> bool {
    let predicates = generics
        .where_clause
        .iter()
        .flat_map(|clause| &clause.predicates);
    generics
        .params
        .iter()
        .any(|param| names_self(param.to_token_stream()))
        || predicates.into_iter().any(predicate_names_self)
}

fn predicate_names_self(predicate: &syn::WherePredicate) -> bool {
    match predicate {
        syn::WherePredicate::Type(predicate) if is_bare_self(unwrapped(&predicate.bounded_ty)) => {
            names_self(predicate.bounds.to_token_stream())
        }
        other => names_self(other.to_token_stream()),
    }
}

/// The strongest bound the `where` clause puts on `Self`: a trait, then a
/// lifetime, then `Sized` alone.
pub(super) fn self_bound(where_clause: Option<&syn::WhereClause>) -> SelfBound {
    let predicates = where_clause.iter().flat_map(|clause| &clause.predicates);
    let bounds: Vec<_> = predicates.filter_map(bounds_on_self).flatten().collect();
    if !bounds.iter().copied().all(is_sized_or_lifetime) {
        SelfBound::Trait
    } else if bounds.iter().copied().any(is_lifetime) {
        SelfBound::Lifetime
    } else {
        SelfBound::Sized
    }
}

fn bounds_on_self(
    predicate: &syn::WherePredicate,
) -> Option<&syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>> {
    match predicate {
        syn::WherePredicate::Type(predicate) if is_bare_self(unwrapped(&predicate.bounded_ty)) => {
            Some(&predicate.bounds)
        }
        _ => None,
    }
}

fn is_lifetime(bound: &syn::TypeParamBound) -> bool {
    matches!(bound, syn::TypeParamBound::Lifetime(_))
}

fn is_sized_or_lifetime(bound: &syn::TypeParamBound) -> bool {
    match bound {
        syn::TypeParamBound::Lifetime(_) => true,
        syn::TypeParamBound::Trait(bound) => {
            matches!(bound.modifier, syn::TraitBoundModifier::None)
                && bound
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "Sized")
        }
        _ => false,
    }
}

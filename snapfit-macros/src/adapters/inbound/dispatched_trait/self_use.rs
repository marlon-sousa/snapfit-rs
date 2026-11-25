//! Adapter: finds where a signature or a bound names `Self` itself, and what
//! a `where` clause bounds `Self` by. Plays the role of its parent.

use syn::visit::{self, Visit};

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

/// The type inside the invisible group a `macro_rules!` puts around `$t:ty`,
/// or inside parentheses.
pub(super) fn unwrapped(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Group(group) => unwrapped(&group.elem),
        syn::Type::Paren(paren) => unwrapped(&paren.elem),
        _ => ty,
    }
}

pub(super) fn is_bare_self(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("Self"))
}

/// `Forwardable` when every bound the `where` clause puts on `Self` is
/// `Sized` or a lifetime.
pub(super) fn self_bound(where_clause: Option<&syn::WhereClause>) -> SelfBound {
    let predicates = where_clause.iter().flat_map(|clause| &clause.predicates);
    if predicates
        .filter_map(bounds_on_self)
        .flatten()
        .all(is_forwardable)
    {
        SelfBound::Forwardable
    } else {
        SelfBound::Other
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

/// `Sized`, which every dispatch type is, or a lifetime, which a dispatch
/// type outlives whenever every implementation it holds does.
fn is_forwardable(bound: &syn::TypeParamBound) -> bool {
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

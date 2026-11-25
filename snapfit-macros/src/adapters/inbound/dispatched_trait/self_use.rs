//! Adapter: finds where a signature or a bound names `Self` itself. Plays the
//! role of its parent.

use syn::visit::{self, Visit};

use crate::domain::SelfUse;

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

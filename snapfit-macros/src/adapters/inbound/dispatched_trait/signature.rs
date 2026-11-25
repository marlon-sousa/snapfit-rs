//! Adapter: reads how a method of a dispatched trait takes `self`, where its
//! signature names `Self`, and what it returns. Plays the role of its parent.

use syn::visit::{self, Visit};

use crate::domain::{MethodShape, Receiver, ReturnShape, SelfUse};

#[cfg(test)]
mod tests;

pub(super) fn method_shape(signature: &syn::Signature) -> MethodShape {
    MethodShape::new(
        receiver(signature),
        self_use(signature),
        return_shape(&signature.output),
    )
}

fn receiver(signature: &syn::Signature) -> Receiver {
    let Some(receiver) = signature.receiver() else {
        return Receiver::None;
    };
    match unwrapped(&receiver.ty) {
        ty if is_bare_self(ty) => Receiver::Value,
        syn::Type::Reference(reference) if is_bare_self(unwrapped(&reference.elem)) => {
            Receiver::Reference
        }
        _ => Receiver::Other,
    }
}

fn return_shape(output: &syn::ReturnType) -> ReturnShape {
    let syn::ReturnType::Type(_, ty) = output else {
        return ReturnShape::Plain;
    };
    if is_bare_self(unwrapped(ty)) {
        return ReturnShape::ItSelf;
    }
    let mut finder = ImplTraitFinder(ReturnShape::Plain);
    finder.visit_type(ty);
    finder.0
}

/// Where `Self` appears outside the receiver, the whole return type, a
/// `Self::` path and a bound on `Self`.
fn self_use(signature: &syn::Signature) -> SelfUse {
    let mut finder = SelfFinder(SelfUse::Forwardable);
    finder.visit_generics(&signature.generics);
    for ty in signature.inputs.iter().filter_map(typed_argument) {
        finder.visit_type(ty);
    }
    if let syn::ReturnType::Type(_, ty) = &signature.output {
        if !is_bare_self(unwrapped(ty)) {
            finder.visit_type(ty);
        }
    }
    finder.0
}

fn typed_argument(input: &syn::FnArg) -> Option<&syn::Type> {
    match input {
        syn::FnArg::Typed(argument) => Some(&argument.ty),
        syn::FnArg::Receiver(_) => None,
    }
}

/// The type inside the invisible group a `macro_rules!` puts around `$t:ty`,
/// or inside parentheses.
fn unwrapped(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Group(group) => unwrapped(&group.elem),
        syn::Type::Paren(paren) => unwrapped(&paren.elem),
        _ => ty,
    }
}

fn is_bare_self(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("Self"))
}

struct SelfFinder(SelfUse);

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

struct ImplTraitFinder(ReturnShape);

impl<'ast> Visit<'ast> for ImplTraitFinder {
    fn visit_type_impl_trait(&mut self, _: &'ast syn::TypeImplTrait) {
        self.0 = ReturnShape::ImplTrait;
    }
}

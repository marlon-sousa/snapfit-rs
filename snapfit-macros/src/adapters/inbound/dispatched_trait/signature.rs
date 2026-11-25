//! Adapter: reads how a method of a dispatched trait takes `self`, where its
//! signature names `Self`, what it bounds `Self` by, and what it returns.
//! Plays the role of its parent.

use syn::visit::Visit;

use super::self_use::{SelfFinder, is_bare_self, self_bound, unwrapped};
use crate::domain::{Execution, MethodShape, Receiver, ReturnShape, Safety, SelfUse};

#[cfg(test)]
mod tests;

pub(super) fn method_shape(signature: &syn::Signature) -> MethodShape {
    MethodShape::new(
        receiver(signature),
        self_use(signature),
        self_bound(signature.generics.where_clause.as_ref()),
        return_shape(&signature.output),
        signature
            .asyncness
            .map_or(Execution::Sync, |_| Execution::Async),
        signature.unsafety.map_or(Safety::Safe, |_| Safety::Unsafe),
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

fn self_use(signature: &syn::Signature) -> SelfUse {
    let mut finder = SelfFinder::new();
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

struct ImplTraitFinder(ReturnShape);

impl<'ast> Visit<'ast> for ImplTraitFinder {
    fn visit_type_impl_trait(&mut self, _: &'ast syn::TypeImplTrait) {
        self.0 = ReturnShape::ImplTrait;
    }
}

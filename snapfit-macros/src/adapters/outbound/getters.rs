//! Adapter: one getter per dependency, which clones a singleton or builds a transient.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

use super::expr::{Site, ValueSource, construct};
use crate::adapters::syntax::{FieldSyntax, Plan};
use crate::domain::Step;

#[cfg(test)]
mod tests;

pub(super) fn emit_getters_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let site = Site {
        container: &plan.syntax,
        source: ValueSource::SelfFields,
    };
    let getters = plan
        .declaration_order
        .iter()
        .map(|step| emit_getter(step, site));

    quote! {
        impl #container_ident {
            #(#getters)*
        }
    }
}

fn emit_getter(step: &Step<FieldSyntax>, site: Site<'_>) -> TokenStream {
    let FieldSyntax { ident, ty, .. } = &step.syntax;
    let getter = format_ident!("get_{}", ident.unraw());
    let (body, doc) = if step.lifecycle.is_singleton() {
        let doc = format!(
            "Returns a clone of `{}`, built once with the container.",
            ident.unraw()
        );
        (quote!(self.#ident.clone()), doc)
    } else {
        let doc = format!("Builds a new `{}` for this call.", ident.unraw());
        (construct(step, site), doc)
    };
    let allow = private_interfaces(step);

    quote! {
        #[doc = #doc]
        #allow
        pub fn #getter(&self) -> #ty {
            #body
        }
    }
}

/// A dispatched type may hold an implementation more private than an item
/// that names it, which rustc reports at the user's own items instead.
pub(super) fn private_interfaces(step: &Step<FieldSyntax>) -> TokenStream {
    if step.dispatch.is_static() {
        quote!(#[allow(private_interfaces)])
    } else {
        TokenStream::new()
    }
}

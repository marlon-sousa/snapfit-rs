//! Adapter: one getter per dependency, which clones a singleton or builds a transient.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

use super::expr::{ValueSource, construct};
use crate::adapters::syntax::{FieldSyntax, Plan};
use crate::domain::Step;

#[cfg(test)]
mod tests;

pub(super) fn emit_getters_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let getters = plan.declaration_order.iter().map(|step| emit_getter(step));

    quote! {
        impl #container_ident {
            #(#getters)*
        }
    }
}

fn emit_getter(step: &Step<FieldSyntax>) -> TokenStream {
    let FieldSyntax { ident, ty, .. } = &step.syntax;
    let getter = format_ident!("get_{}", ident.unraw());
    let body = if step.lifecycle.is_singleton() {
        quote!(self.#ident.clone())
    } else {
        construct(step, ValueSource::SelfFields)
    };

    quote! {
        pub fn #getter(&self) -> #ty {
            #body
        }
    }
}

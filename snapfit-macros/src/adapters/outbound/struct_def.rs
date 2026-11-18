//! Adapter: the user's struct written back, without snapfit's attributes or transient fields.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::syntax::{ContainerSyntax, FieldSyntax, Plan};
use crate::domain::Step;

#[cfg(test)]
mod tests;

/// Keeps the attributes, docs, visibility and field order as written. A
/// transient is built on demand and needs no storage.
pub(super) fn emit_struct_definition(plan: &Plan) -> TokenStream {
    let ContainerSyntax {
        ident, vis, attrs, ..
    } = &plan.syntax;
    let fields = plan
        .declaration_order
        .iter()
        .filter(|step| step.lifecycle.is_singleton())
        .map(|step| emit_field(step));

    quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#fields,)*
        }
    }
}

fn emit_field(step: &Step<FieldSyntax>) -> TokenStream {
    let FieldSyntax {
        ident,
        ty,
        vis,
        attrs,
        ..
    } = &step.syntax;
    // A singleton may only feed other dependencies and never be read through
    // its getter; that is normal for a container, not dead code.
    quote!(#(#attrs)* #[allow(dead_code)] #vis #ident: #ty)
}

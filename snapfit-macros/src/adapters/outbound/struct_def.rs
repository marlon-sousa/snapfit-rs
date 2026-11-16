use proc_macro2::TokenStream;
use quote::quote;

use crate::domain::ConstructionPlan;
use crate::ports::{ContainerSyntax, FieldSyntax};

#[cfg(test)]
mod tests;

/// Re-emits the user's struct as written (attributes, docs, visibility, field
/// order), minus snapfit's own attributes and the transient fields, which are
/// built on demand and need no storage.
pub(super) fn emit_struct_definition(
    plan: &ConstructionPlan,
    syntax: &ContainerSyntax,
) -> TokenStream {
    let ContainerSyntax {
        ident, vis, attrs, ..
    } = syntax;
    let fields = syntax
        .fields
        .iter()
        .filter(|field| is_stored(plan, field))
        .map(emit_field);

    quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#fields,)*
        }
    }
}

fn is_stored(plan: &ConstructionPlan, field: &FieldSyntax) -> bool {
    plan.ordered_steps
        .iter()
        .any(|step| field.ident == step.dep_name.as_str() && step.lifecycle.is_singleton())
}

fn emit_field(field: &FieldSyntax) -> TokenStream {
    let FieldSyntax {
        ident,
        ty,
        vis,
        attrs,
        ..
    } = field;
    // A singleton may only feed other dependencies and never be read through
    // its getter; that is normal for a container, not dead code.
    quote!(#(#attrs)* #[allow(dead_code)] #vis #ident: #ty)
}

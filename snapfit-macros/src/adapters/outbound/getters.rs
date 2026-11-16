use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

use super::expr::{ExprBuilder, ValueSource};
use crate::domain::{ConstructionPlan, InstantiationStep};
use crate::ports::ContainerSyntax;

#[cfg(test)]
mod tests;

pub(super) fn emit_getters_impl(plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream {
    let container_ident = &syntax.ident;
    let builder = ExprBuilder::new(plan, syntax, ValueSource::SelfFields);
    let getters = plan
        .ordered_steps
        .iter()
        .map(|step| emit_getter(syntax, &builder, step));

    quote! {
        impl #container_ident {
            #(#getters)*
        }
    }
}

fn emit_getter(
    syntax: &ContainerSyntax,
    builder: &ExprBuilder<'_>,
    step: &InstantiationStep,
) -> TokenStream {
    let field = syntax
        .field(step.dep_name.as_str())
        .expect("every planned dependency comes from a named container field");
    let getter = format_ident!("get_{}", field.ident.unraw());
    let ty = &field.ty;
    let ident = &field.ident;
    let body = if step.lifecycle.is_singleton() {
        quote!(self.#ident.clone())
    } else {
        builder.construct(step)
    };

    quote! {
        pub fn #getter(&self) -> #ty {
            #body
        }
    }
}

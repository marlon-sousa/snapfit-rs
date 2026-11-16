use proc_macro2::TokenStream;
use quote::quote;

use super::expr::{ExprBuilder, ValueSource, local_ident};
use crate::domain::{ConstructionPlan, InstantiationStep};
use crate::ports::ContainerSyntax;

#[cfg(test)]
mod tests;

pub(super) fn emit_init_impl(plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream {
    if plan.is_async {
        emit_async_init_impl(plan, syntax)
    } else {
        emit_sync_new_impl(plan, syntax)
    }
}

fn emit_sync_new_impl(plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream {
    let container_ident = &syntax.ident;
    let body = emit_construction_body(plan, syntax);

    quote! {
        impl #container_ident {
            pub fn new() -> Self {
                #body
            }
        }

        impl ::std::default::Default for #container_ident {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ::snapfit::Container for #container_ident {
            fn new() -> Self {
                Self::new()
            }
        }
    }
}

fn emit_async_init_impl(plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream {
    let container_ident = &syntax.ident;
    let body = emit_construction_body(plan, syntax);

    quote! {
        impl #container_ident {
            pub async fn init() -> ::std::result::Result<
                Self,
                ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync>,
            > {
                ::std::result::Result::Ok({ #body })
            }
        }
    }
}

/// Binds every singleton in dependency order, then assembles `Self`.
fn emit_construction_body(plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream {
    let builder = ExprBuilder::new(plan, syntax, ValueSource::Locals);
    let singletons: Vec<&InstantiationStep> = plan
        .ordered_steps
        .iter()
        .filter(|step| step.lifecycle.is_singleton())
        .collect();
    let bindings = singletons.iter().map(|step| {
        let field = syntax
            .field(step.dep_name.as_str())
            .expect("every planned dependency comes from a named container field");
        let (local, ty) = (local_ident(&field.ident), &field.ty);
        let value = builder.construct(step);
        quote!(let #local: #ty = #value;)
    });
    let field_inits = singletons
        .iter()
        .filter_map(|step| syntax.field(step.dep_name.as_str()))
        .map(|field| {
            let (ident, local) = (&field.ident, local_ident(&field.ident));
            quote!(#ident: #local)
        });

    quote! {
        #(#bindings)*
        Self {
            #(#field_inits,)*
        }
    }
}

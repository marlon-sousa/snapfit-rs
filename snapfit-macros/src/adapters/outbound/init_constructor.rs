//! Adapter: the container's `new()` and its impls, or its async `init()`.

use proc_macro2::TokenStream;
use quote::quote;

use super::expr::{Site, ValueSource, construct, local_ident};
use crate::adapters::syntax::{FieldSyntax, Plan};
use crate::domain::{Execution, Step};

#[cfg(test)]
mod tests;

pub(super) fn emit_init_impl(plan: &Plan) -> TokenStream {
    match plan.execution {
        Execution::Sync => emit_sync_new_impl(plan),
        Execution::Async => emit_async_init_impl(plan),
    }
}

fn emit_sync_new_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let body = emit_construction_body(plan);

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

fn emit_async_init_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let body = emit_construction_body(plan);

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

/// Binds every singleton in construction order, then assembles `Self`.
fn emit_construction_body(plan: &Plan) -> TokenStream {
    let site = Site {
        container: &plan.syntax.ident,
        source: ValueSource::Locals,
    };
    let bindings = plan.singletons().map(|step| binding(step, site));
    let field_inits = plan.singletons().map(field_init);

    quote! {
        #(#bindings)*
        Self {
            #(#field_inits,)*
        }
    }
}

/// `let __snapfit_db: Arc<Database> = ...;`
fn binding(step: &Step<FieldSyntax>, site: Site<'_>) -> TokenStream {
    let local = local_ident(&step.syntax.ident);
    let ty = &step.syntax.ty;
    let value = construct(step, site);
    quote!(let #local: #ty = #value;)
}

/// `db: __snapfit_db`
fn field_init(step: &Step<FieldSyntax>) -> TokenStream {
    let ident = &step.syntax.ident;
    let local = local_ident(ident);
    quote!(#ident: #local)
}

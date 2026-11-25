//! Adapter: the container's `new()` and its impls, its `try_new()` and its
//! impl when it can fail, or its async `init()`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;

use super::expr::{Site, ValueSource, construct, local_ident};
use crate::adapters::syntax::{FieldSyntax, Plan};
use crate::domain::{Execution, Fallibility, Step};

#[cfg(test)]
mod tests;

pub(super) fn emit_init_impl(plan: &Plan) -> TokenStream {
    match (plan.execution, plan.fallibility) {
        (Execution::Sync, Fallibility::Infallible) => emit_sync_new_impl(plan),
        (Execution::Sync, Fallibility::Fallible) => emit_sync_try_new_impl(plan),
        (Execution::Async, _) => emit_async_init_impl(plan),
    }
}

fn emit_sync_new_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let body = emit_construction_body(plan);
    let doc = format!(
        "Builds `{}` with each of its singletons, in the order their dependencies need.",
        container_ident.unraw()
    );

    quote! {
        impl #container_ident {
            #[doc = #doc]
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

/// `try_new()`, which returns the first field's error; the container has no
/// `new()`, `Default` or `Container`, which promise that building it succeeds.
fn emit_sync_try_new_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let body = emit_construction_body(plan);
    let doc = format!(
        "Builds `{}` with each of its singletons, or returns the `snapfit::BuildError` of the first field that fails.",
        container_ident.unraw()
    );

    quote! {
        impl #container_ident {
            #[doc = #doc]
            pub fn try_new() -> ::std::result::Result<Self, ::snapfit::BuildError> {
                ::std::result::Result::Ok({ #body })
            }
        }

        impl ::snapfit::FallibleContainer for #container_ident {
            fn try_new() -> ::std::result::Result<Self, ::snapfit::BuildError> {
                Self::try_new()
            }
        }
    }
}

fn emit_async_init_impl(plan: &Plan) -> TokenStream {
    let container_ident = &plan.syntax.ident;
    let body = emit_construction_body(plan);
    let doc = format!(
        "Builds `{}` with each of its singletons, awaiting those built asynchronously, or returns the first error.",
        container_ident.unraw()
    );

    quote! {
        impl #container_ident {
            #[doc = #doc]
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
        container: &plan.syntax,
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

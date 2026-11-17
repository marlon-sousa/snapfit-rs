//! Adapter: writes a construction plan, or the problems found, as Rust tokens through quote.

mod diagnostics;
mod expr;
mod getters;
mod init_constructor;
mod struct_def;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::syntax::Plan;

pub(crate) use diagnostics::{Origins, fallback_struct};

/// `#[container]` rewrites the struct and adds its impls.
pub(crate) fn emit_container(plan: &Plan) -> TokenStream {
    let struct_def = struct_def::emit_struct_definition(plan);
    let impls = emit_derive_impls(plan);

    quote! {
        #struct_def
        #impls
    }
}

/// `#[derive(Container)]` cannot rewrite the struct, so it only adds impls.
pub(crate) fn emit_derive_impls(plan: &Plan) -> TokenStream {
    let init_impl = init_constructor::emit_init_impl(plan);
    let getters = getters::emit_getters_impl(plan);

    quote! {
        #init_impl
        #getters
    }
}

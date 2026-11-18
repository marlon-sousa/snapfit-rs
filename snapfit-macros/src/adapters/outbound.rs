//! Adapter: writes a construction plan, or the problems found, as Rust tokens through quote.

mod diagnostics;
mod expr;
mod factories;
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
    let impls = emit_impls(plan);

    quote! {
        #struct_def
        #impls
    }
}

/// The constructor, the `Default` and `Container` impls, the getters, and the
/// factories trait with its inputs structs.
fn emit_impls(plan: &Plan) -> TokenStream {
    let init_impl = init_constructor::emit_init_impl(plan);
    let getters = getters::emit_getters_impl(plan);
    let factories = factories::emit_factories(plan);

    quote! {
        #init_impl
        #getters
        #factories
    }
}

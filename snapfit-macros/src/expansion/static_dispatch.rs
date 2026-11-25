//! Handler: turns `#[snapfit::static_dispatch]`'s tokens into the dispatch
//! attribute, has it planned, and writes the trait with its impls, or with
//! every problem found. Plays the role of its parent.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::{TraitOrigins, emit_dispatch_impls, read_dispatch_attribute};
use crate::use_cases::plan_dispatch;

#[cfg(test)]
mod tests;

/// The item is written back as the user wrote it in every case, so that a
/// refused trait leaves no follow-on "cannot find trait" errors.
pub(crate) fn static_dispatch_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = match syn::parse2::<syn::Item>(input) {
        Ok(item) => item,
        Err(error) => return error.to_compile_error(),
    };
    match read_dispatch_attribute(&args, &item).and_then(plan_dispatch) {
        Ok(plan) => {
            let impls = emit_dispatch_impls(&plan);
            quote!(#item #impls)
        }
        Err(errors) => {
            let errors = TraitOrigins::new(&args, &item).compile_errors(&errors);
            quote!(#errors #item)
        }
    }
}

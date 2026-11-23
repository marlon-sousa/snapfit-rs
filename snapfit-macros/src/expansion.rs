//! Handler: turns a macro's tokens into the declaration, has it planned, and
//! turns the plan, or every problem found, back into tokens.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::{Origins, emit_container, fallback_struct, parse_attribute};
use crate::domain::FeatureGate;
use crate::use_cases::plan_container;

mod impl_fields;
#[cfg(test)]
mod tests;

const BUILD_FEATURE_GATE: FeatureGate = if cfg!(feature = "unstable") {
    FeatureGate::Unstable
} else {
    FeatureGate::Stable
};

pub(crate) fn container_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    expand_attribute(args, input, BUILD_FEATURE_GATE)
}

fn expand_attribute(args: TokenStream, input: TokenStream, gate: FeatureGate) -> TokenStream {
    let mut item = match syn::parse2::<syn::ItemStruct>(input) {
        Ok(item) => item,
        Err(error) => return error.to_compile_error(),
    };
    let mentions = impl_fields::rewrite(&mut item);
    let plan =
        parse_attribute(args, &item).and_then(|declaration| plan_container(declaration, gate));
    match plan {
        Ok(plan) => {
            let container = emit_container(&plan);
            quote!(#container #mentions)
        }
        Err(errors) => {
            let errors = Origins::new(&item.ident, &item.fields).compile_errors(&errors);
            let fallback = fallback_struct(&item);
            quote!(#errors #fallback)
        }
    }
}

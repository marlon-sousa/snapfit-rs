//! Handler: turns a macro's tokens into the declaration, has it planned, and
//! turns the plan, or every problem found, back into tokens: `#[container]`
//! here, `#[snapfit::static_dispatch]` in its submodule.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::{Origins, emit_aliases, emit_container, fallback_struct, parse_attribute};
use crate::domain::FeatureGate;
use crate::use_cases::{declared_dispatched_types, plan_container};

mod static_dispatch;

#[cfg(test)]
mod tests;

pub(crate) use static_dispatch::static_dispatch_attribute;

const BUILD_FEATURE_GATE: FeatureGate = if cfg!(feature = "unstable") {
    FeatureGate::Unstable
} else {
    FeatureGate::Stable
};

pub(crate) fn container_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    expand_attribute(args, input, BUILD_FEATURE_GATE)
}

fn expand_attribute(args: TokenStream, input: TokenStream, gate: FeatureGate) -> TokenStream {
    let item = match syn::parse2::<syn::ItemStruct>(input) {
        Ok(item) => item,
        Err(error) => return error.to_compile_error(),
    };
    let declaration = parse_attribute(args, &item);
    let declared = declaration
        .as_ref()
        .map(declared_dispatched_types)
        .unwrap_or_default();
    match declaration.and_then(|declaration| plan_container(declaration, gate)) {
        Ok(plan) => emit_container(&plan),
        Err(errors) => {
            let errors = Origins::new(&item).compile_errors(&errors);
            let fallback = fallback_struct(&item);
            let aliases = emit_aliases(&declared, &item.vis);
            quote!(#errors #fallback #aliases)
        }
    }
}

//! Handler: turns a macro's tokens into the declaration, has it planned, and
//! turns the plan, or every problem found, back into tokens: `#[container]`
//! here, `#[snapfit::static_dispatch]` in its submodule.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::{
    Origins, emit_container, emit_fallback_aliases, fallback_struct, parse_attribute,
    read_readable_parts, readable_runtime_path,
};
use crate::domain::{DomainErrors, FeatureGate};
use crate::use_cases::{fallback_dispatched_types, plan_container};

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
    let runtime = readable_runtime_path(&args);
    match parse_attribute(args, &item) {
        Ok(declaration) => plan_container(declaration, gate).map_or_else(
            |errors| refused(&item, &errors, &runtime),
            |plan| emit_container(&plan),
        ),
        Err(errors) => refused(&item, &errors, &runtime),
    }
}

/// Every error, the struct as written, and the alias of each type its
/// readable dispatch attributes declare, holding what the parts that read
/// give it, so that the fields that name it raise no error of their own;
/// they reach snapfit through whatever `crate` path the arguments give.
fn refused(item: &syn::ItemStruct, errors: &DomainErrors, runtime: &syn::Path) -> TokenStream {
    let errors = Origins::new(item).compile_errors(errors);
    let fallback = fallback_struct(item);
    let parts = read_readable_parts(item);
    let types = fallback_dispatched_types(&parts.declarations, &parts.extensions, &parts.fields);
    let aliases = emit_fallback_aliases(&types, &item.vis, runtime);
    quote!(#errors #fallback #aliases)
}

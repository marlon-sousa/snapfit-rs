//! Handler: turns a macro's tokens into the declaration, has it planned, and
//! turns the plan, or every problem found, back into tokens: `#[container]`
//! here, `#[snapfit::static_dispatch]` in its submodule.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::{
    Origins, emit_container, emit_fallback_aliases, fallback_struct, parse_attribute,
    read_readable_dispatch,
};
use crate::domain::{DependencyDeclaration, DomainErrors, FeatureGate};
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
    match parse_attribute(args, &item) {
        Ok(declaration) => {
            let aliases = fallback_aliases(&item, &declaration.dependencies);
            plan_container(declaration, gate).map_or_else(
                |errors| refused(&item, &errors, aliases),
                |plan| emit_container(&plan),
            )
        }
        Err(errors) => refused(&item, &errors, fallback_aliases::<()>(&item, &[])),
    }
}

/// The aliases of the types the struct's readable dispatch attributes
/// declare, holding what the fields that could be read name.
fn fallback_aliases<F>(
    item: &syn::ItemStruct,
    dependencies: &[DependencyDeclaration<F, syn::Type>],
) -> TokenStream {
    let (declarations, extensions) = read_readable_dispatch(item);
    let types = fallback_dispatched_types(&declarations, &extensions, dependencies);
    emit_fallback_aliases(&types, &item.vis)
}

/// Every error, the struct as written, and the aliases its fields name.
fn refused(item: &syn::ItemStruct, errors: &DomainErrors, aliases: TokenStream) -> TokenStream {
    let errors = Origins::new(item).compile_errors(errors);
    let fallback = fallback_struct(item);
    quote!(#errors #fallback #aliases)
}

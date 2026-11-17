//! Handler: turns a macro's tokens into the declaration, has it planned, and
//! turns the plan, or every problem found, back into tokens.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::{
    Origins, emit_container, emit_derive_impls, fallback_struct, parse_attribute, parse_derive,
};
use crate::use_cases::{FeatureGate, plan_container};

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

pub(crate) fn container_derive(input: TokenStream) -> TokenStream {
    expand_derive(input, BUILD_FEATURE_GATE)
}

fn expand_attribute(args: TokenStream, input: TokenStream, gate: FeatureGate) -> TokenStream {
    let item = match syn::parse2::<syn::ItemStruct>(input) {
        Ok(item) => item,
        Err(error) => return error.to_compile_error(),
    };
    let plan =
        parse_attribute(args, &item).and_then(|declaration| plan_container(declaration, gate));
    match plan {
        Ok(plan) => emit_container(&plan),
        Err(errors) => {
            let errors = Origins::new(&item.ident, &item.fields).compile_errors(&errors);
            let fallback = fallback_struct(&item);
            quote!(#errors #fallback)
        }
    }
}

fn expand_derive(input: TokenStream, gate: FeatureGate) -> TokenStream {
    let input = match syn::parse2::<syn::DeriveInput>(input) {
        Ok(input) => input,
        Err(error) => return error.to_compile_error(),
    };
    let plan = parse_derive(&input).and_then(|declaration| plan_container(declaration, gate));
    match plan {
        Ok(plan) => emit_derive_impls(&plan),
        Err(errors) => {
            let fields = match &input.data {
                syn::Data::Struct(data) => data.fields.clone(),
                syn::Data::Enum(_) | syn::Data::Union(_) => syn::Fields::Unit,
            };
            Origins::new(&input.ident, &fields).compile_errors(&errors)
        }
    }
}

#![deny(warnings)]

mod adapters;
mod domain;
mod ports;
mod use_cases;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

use crate::adapters::{QuoteEmitterAdapter, SynParserAdapter};
use crate::domain::{ContainerIR, DomainError};
use crate::ports::{CodeEmitterPort, ContainerParserPort};
use crate::use_cases::{DeriveContainerUseCase, FeatureGate};

const BUILD_FEATURE_GATE: FeatureGate = FeatureGate::new(cfg!(feature = "unstable"));

#[proc_macro_attribute]
pub fn container(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);
    expand_container_attribute(args.into(), &item_struct, BUILD_FEATURE_GATE)
        .unwrap_or_else(to_compile_error)
        .into()
}

#[proc_macro_derive(Container, attributes(container, singleton, transient, dispatch))]
pub fn derive_container(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    expand_derive_container(&derive_input, BUILD_FEATURE_GATE)
        .unwrap_or_else(to_compile_error)
        .into()
}

fn expand_container_attribute(
    args: proc_macro2::TokenStream,
    item_struct: &syn::ItemStruct,
    feature_gate: FeatureGate,
) -> Result<proc_macro2::TokenStream, DomainError> {
    let parser = SynParserAdapter::new();
    let ir = if args.is_empty() {
        parser.parse_container(item_struct)?
    } else {
        parser.parse_with_args(args, item_struct)?
    };
    feature_gate.check(&ir)?;
    let plan = DeriveContainerUseCase::new().execute(ir)?;
    Ok(QuoteEmitterAdapter::new().emit_container(&plan))
}

fn expand_derive_container(
    derive_input: &syn::DeriveInput,
    feature_gate: FeatureGate,
) -> Result<proc_macro2::TokenStream, DomainError> {
    let ir = SynParserAdapter::new().parse_derive_input(derive_input)?;
    validate_derive_compatibility(&ir)?;
    feature_gate.check(&ir)?;
    let plan = DeriveContainerUseCase::new().execute(ir)?;
    Ok(QuoteEmitterAdapter::new().emit_derive_impls(&plan))
}

fn validate_derive_compatibility(ir: &ContainerIR) -> Result<(), DomainError> {
    let requires_struct_rewrite = ir
        .dependencies
        .iter()
        .any(|dep| dep.lifecycle.is_transient() || dep.trait_dispatch.is_enum());

    (!requires_struct_rewrite).then_some(()).ok_or_else(|| {
        DomainError::InvalidConfiguration(
            "#[derive(Container)] does not support #[transient] or #[dispatch] fields; use #[container] instead".to_string(),
        )
    })
}

fn to_compile_error(err: DomainError) -> proc_macro2::TokenStream {
    syn::Error::new(proc_macro2::Span::call_site(), err).to_compile_error()
}

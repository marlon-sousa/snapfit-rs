#![deny(warnings)]

mod adapters;
mod domain;
mod ports;
mod use_cases;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;
use quote::quote;

use crate::adapters::{QuoteEmitterAdapter, SynParserAdapter};
use crate::domain::{ContainerIR, DomainError};
use crate::ports::{CodeEmitterPort, ContainerParserPort, ContainerSyntax};
use crate::use_cases::{DeriveContainerUseCase, FeatureGate};

const BUILD_FEATURE_GATE: FeatureGate = FeatureGate::new(cfg!(feature = "unstable"));

#[proc_macro_attribute]
pub fn container(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);
    let syntax = syntax_of_struct(&item_struct);
    expand_container_attribute(args.into(), &item_struct, &syntax, BUILD_FEATURE_GATE)
        .unwrap_or_else(|err| {
            let error = to_compile_error(&syntax, err);
            let fallback = fallback_struct(&syntax);
            quote!(#error #fallback)
        })
        .into()
}

#[proc_macro_derive(Container, attributes(container, singleton, transient, dispatch))]
pub fn derive_container(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);
    let syntax = syntax_of_derive_input(&derive_input);
    expand_derive_container(&derive_input, &syntax, BUILD_FEATURE_GATE)
        .unwrap_or_else(|err| to_compile_error(&syntax, err))
        .into()
}

fn expand_container_attribute(
    args: proc_macro2::TokenStream,
    item_struct: &syn::ItemStruct,
    syntax: &ContainerSyntax,
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
    Ok(QuoteEmitterAdapter::new().emit_container(&plan, syntax))
}

fn expand_derive_container(
    derive_input: &syn::DeriveInput,
    syntax: &ContainerSyntax,
    feature_gate: FeatureGate,
) -> Result<proc_macro2::TokenStream, DomainError> {
    let ir = SynParserAdapter::new().parse_derive_input(derive_input)?;
    validate_derive_compatibility(&ir)?;
    feature_gate.check(&ir)?;
    let plan = DeriveContainerUseCase::new().execute(ir)?;
    Ok(QuoteEmitterAdapter::new().emit_derive_impls(&plan, syntax))
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

fn syntax_of_struct(item: &syn::ItemStruct) -> ContainerSyntax {
    ContainerSyntax::new(
        &item.ident,
        &item.vis,
        &item.attrs,
        &item.generics,
        &item.fields,
    )
}

fn syntax_of_derive_input(input: &syn::DeriveInput) -> ContainerSyntax {
    let unit = syn::Fields::Unit;
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => &unit,
    };
    ContainerSyntax::new(
        &input.ident,
        &input.vis,
        &input.attrs,
        &input.generics,
        fields,
    )
}

/// Points the diagnostic at the field it is about, or at the container.
fn to_compile_error(syntax: &ContainerSyntax, err: DomainError) -> proc_macro2::TokenStream {
    syntax.error_for(err).to_compile_error()
}

/// Keeps the struct (without snapfit attributes) when expansion fails, so the
/// user sees the real error instead of follow-on "cannot find type" errors.
fn fallback_struct(syntax: &ContainerSyntax) -> proc_macro2::TokenStream {
    let ContainerSyntax {
        ident,
        vis,
        attrs,
        generics,
        fields,
    } = syntax;
    let where_clause = &generics.where_clause;
    let fields = fields.iter().map(|field| {
        let (attrs, vis, ident, ty) = (&field.attrs, &field.vis, &field.ident, &field.ty);
        quote!(#(#attrs)* #vis #ident: #ty)
    });
    quote! {
        #(#attrs)*
        #vis struct #ident #generics #where_clause {
            #(#fields,)*
        }
    }
}

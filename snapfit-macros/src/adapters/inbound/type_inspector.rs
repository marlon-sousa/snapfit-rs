use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::domain::{DomainError, TraitDispatch};

#[cfg(test)]
mod tests;

pub(super) fn compact_type_tokens(tokens: TokenStream) -> String {
    let mut text = tokens
        .to_string()
        .replace(" :: ", "::")
        .replace(" < ", "<")
        .replace(" <", "<")
        .replace("< ", "<")
        .replace(" , ", ", ")
        .replace("& ", "&")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" (", "(");
    while text.contains(" >") {
        text = text.replace(" >", ">");
    }
    text.trim().to_string()
}

pub(super) fn parse_dispatch_attribute(attr: &syn::Attribute) -> Result<Vec<String>, DomainError> {
    let syn::Meta::List(list) = &attr.meta else {
        return Err(DomainError::InvalidConfiguration(
            "#[dispatch(...)] requires a parenthesized list of implementation types".to_string(),
        ));
    };
    let parsed = Punctuated::<syn::Type, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .map_err(|err| DomainError::InvalidConfiguration(err.to_string()))?;
    let impls: Vec<String> = parsed
        .iter()
        .map(|ty| compact_type_tokens(quote!(#ty)))
        .collect();
    (!impls.is_empty()).then_some(impls).ok_or_else(|| {
        DomainError::InvalidConfiguration(
            "#[dispatch(...)] requires at least one implementation type".to_string(),
        )
    })
}

pub(super) fn resolve_trait_dispatch(
    ty: &syn::Type,
    dispatch_impls: Option<Vec<String>>,
) -> Result<TraitDispatch, DomainError> {
    if let Some(impls) = dispatch_impls {
        let trait_name = extract_trait_name(ty)?;
        return Ok(TraitDispatch::Enum { trait_name, impls });
    }

    Ok(
        find_trait_object(ty).map_or(TraitDispatch::None, |trait_obj| TraitDispatch::Dyn {
            trait_type: compact_type_tokens(quote!(#trait_obj)),
        }),
    )
}

fn extract_trait_name(ty: &syn::Type) -> Result<String, DomainError> {
    if let Some(trait_obj) = find_trait_object(ty) {
        return trait_obj
            .bounds
            .iter()
            .find_map(extract_primary_trait_bound)
            .ok_or_else(|| {
                DomainError::InvalidConfiguration(
                    "trait object in #[dispatch] must specify a primary trait bound".to_string(),
                )
            });
    }

    Err(DomainError::InvalidConfiguration(
        "#[dispatch(...)] field type must contain a 'dyn Trait' trait object".to_string(),
    ))
}

fn extract_primary_trait_bound(bound: &syn::TypeParamBound) -> Option<String> {
    let syn::TypeParamBound::Trait(trait_bound) = bound else {
        return None;
    };
    if matches!(trait_bound.modifier, syn::TraitBoundModifier::Maybe(_)) {
        return None;
    }
    let path = &trait_bound.path;
    let is_auto_marker = path.is_ident("Send") || path.is_ident("Sync") || path.is_ident("Unpin");
    (!is_auto_marker).then(|| compact_type_tokens(quote!(#path)))
}

fn find_trait_object(ty: &syn::Type) -> Option<&syn::TypeTraitObject> {
    match ty {
        syn::Type::TraitObject(trait_obj) => Some(trait_obj),
        syn::Type::Reference(r) => find_trait_object(&r.elem),
        syn::Type::Paren(p) => find_trait_object(&p.elem),
        syn::Type::Group(g) => find_trait_object(&g.elem),
        syn::Type::Path(type_path) => {
            type_path
                .path
                .segments
                .last()
                .and_then(|seg| match &seg.arguments {
                    syn::PathArguments::AngleBracketed(args) => args.args.iter().find_map(|arg| {
                        let syn::GenericArgument::Type(inner_ty) = arg else {
                            return None;
                        };
                        find_trait_object(inner_ty)
                    }),
                    _ => None,
                })
        }
        _ => None,
    }
}

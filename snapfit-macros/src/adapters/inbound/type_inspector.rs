//! Adapter: what the field's syn type says about building it, as facts for the domain.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::domain::{AttributeProblem, TypeShape};

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

pub(super) fn parse_dispatch_attribute(
    attr: &syn::Attribute,
) -> Result<Vec<String>, AttributeProblem> {
    let syn::Meta::List(list) = &attr.meta else {
        return Err(AttributeProblem::DispatchWithoutList);
    };
    let parsed = Punctuated::<syn::Type, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .map_err(|err| AttributeProblem::Unreadable(err.to_string()))?;
    let impls: Vec<String> = parsed
        .iter()
        .map(|ty| compact_type_tokens(quote!(#ty)))
        .collect();
    (!impls.is_empty())
        .then_some(impls)
        .ok_or(AttributeProblem::DispatchWithoutImplementations)
}

/// What building a value of `ty` has to know: whether it holds a trait
/// object, which has no constructor, or is an `Arc`.
pub(super) fn type_shape(ty: &syn::Type) -> TypeShape {
    match (find_trait_object(ty), arc_inner_type(ty)) {
        (Some(_), _) => TypeShape::TraitObject,
        (None, Some(_)) => TypeShape::Arc,
        (None, None) => TypeShape::Value,
    }
}

/// `T` when `ty` is `Arc<T>`, `std::sync::Arc<T>`, `::std::sync::Arc<T>` or
/// `alloc::sync::Arc<T>`.
pub(super) fn arc_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    if type_path.qself.is_some() {
        return None;
    }
    let segments: Vec<_> = type_path.path.segments.iter().collect();
    let last = segments.last()?;
    let is_arc_path = last.ident == "Arc"
        && match segments.as_slice() {
            [_] => type_path.path.leading_colon.is_none(),
            [.., sync, _] => sync.ident == "sync",
            [] => false,
        };
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    match (is_arc_path, args.args.first(), args.args.len()) {
        (true, Some(syn::GenericArgument::Type(inner)), 1) => Some(inner),
        _ => None,
    }
}

/// The trait a `#[dispatch(...)]` field's `dyn Trait` names, without its
/// auto-trait bounds.
pub(super) fn dispatched_trait(ty: &syn::Type) -> Result<String, AttributeProblem> {
    find_trait_object(ty)
        .ok_or(AttributeProblem::DispatchWithoutTraitObject)?
        .bounds
        .iter()
        .find_map(extract_primary_trait_bound)
        .ok_or(AttributeProblem::DispatchWithoutTrait)
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

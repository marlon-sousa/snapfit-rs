//! Adapter: what the field's syn type says about building it, as facts for the domain.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::domain::{AttributeProblem, TypeShape};

use trait_objects::{Depth, find_trait_object, trait_object_depth};

mod trait_objects;

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

/// What building a value of `ty` has to know: whether its value, once the
/// field's own `Arc` is set aside, is a trait object, which has no
/// constructor, or only keeps one deeper in its type arguments.
pub(super) fn type_shape(ty: &syn::Type) -> TypeShape {
    let arc_inner = arc_inner_type(ty);
    let value = arc_inner.unwrap_or(ty);
    match (arc_inner, trait_object_depth(value)) {
        (Some(_), Depth::Itself) => TypeShape::ArcTraitObject,
        (_, Depth::Itself | Depth::Argument) => TypeShape::TraitObject,
        (Some(_), Depth::Deeper) => TypeShape::ArcKeepsTraitObject,
        (None, Depth::Deeper) => TypeShape::KeepsTraitObject,
        (Some(_), Depth::Nowhere) => TypeShape::Arc,
        (None, Depth::Nowhere) => TypeShape::Value,
    }
}

/// `T` when `ty` is `Arc<T>`, `std::sync::Arc<T>`, `::std::sync::Arc<T>` or
/// `alloc::sync::Arc<T>`, also when a macro wraps it.
pub(super) fn arc_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    match ty {
        syn::Type::Path(type_path) => arc_path_inner_type(type_path),
        syn::Type::Group(group) => arc_inner_type(&group.elem),
        _ => None,
    }
}

fn arc_path_inner_type(type_path: &syn::TypePath) -> Option<&syn::Type> {
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

//! Adapter: what the field's syn type says about building it, as facts for the domain.

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;

use crate::domain::{TypeName, TypeShape, WrittenType};

use trait_objects::{Depth, trait_object_depth};

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

/// How the type of the field's value, once its own `Arc` is set aside, is
/// written, also when a macro wraps it.
pub(super) fn written_type(ty: &syn::Type) -> WrittenType {
    written_value(arc_inner_type(ty).unwrap_or(ty))
}

fn written_value(ty: &syn::Type) -> WrittenType {
    match ty {
        syn::Type::Path(type_path) => written_path(type_path),
        syn::Type::Group(group) => written_value(&group.elem),
        _ => WrittenType::Other,
    }
}

fn written_path(type_path: &syn::TypePath) -> WrittenType {
    let path = &type_path.path;
    let Some(last) = path.segments.last() else {
        return WrittenType::Other;
    };
    if type_path.qself.is_some() || !last.arguments.is_none() {
        return WrittenType::Other;
    }
    let Ok(name) = TypeName::new(last.ident.to_string()) else {
        return WrittenType::Other;
    };
    if path.segments.len() == 1 && path.leading_colon.is_none() {
        WrittenType::Name(name)
    } else {
        WrittenType::Path {
            last: name,
            written: compact_type_tokens(quote!(#path)),
        }
    }
}

/// A type the container can call a constructor of: not a trait object,
/// and with no `impl Trait` or `_` anywhere in it, which generated code
/// cannot name.
pub(super) fn is_buildable(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Group(group) => is_buildable(&group.elem),
        syn::Type::Paren(paren) => is_buildable(&paren.elem),
        syn::Type::TraitObject(_) => false,
        _ => names_every_type(quote!(#ty)),
    }
}

fn names_every_type(tokens: TokenStream) -> bool {
    tokens.into_iter().all(names_a_type)
}

fn names_a_type(tree: TokenTree) -> bool {
    match tree {
        TokenTree::Ident(word) => word != "impl" && word != "_",
        TokenTree::Group(group) => names_every_type(group.stream()),
        TokenTree::Punct(_) | TokenTree::Literal(_) => true,
    }
}

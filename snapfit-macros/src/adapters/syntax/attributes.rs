//! Adapter: which attributes the user wrote, and what of one decides whether
//! its item exists. Plays the role of its parent.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{Token, parse_quote};

/// The field attributes snapfit reads, and removes from the struct it writes.
pub(crate) fn is_snapfit_attribute(attr: &syn::Attribute) -> bool {
    ["singleton", "transient", "dispatch"]
        .iter()
        .any(|name| attr.path().is_ident(name))
}

/// An attribute snapfit writes back as the user wrote it.
pub(crate) fn is_user_attribute(attr: &&syn::Attribute) -> bool {
    !is_snapfit_attribute(attr) && !is_container_attribute(attr) && !is_dispatch_attribute(attr)
}

/// `static_dispatch_for` or `extend_static_dispatch`, which `#[container]`
/// reads and removes from the struct it writes.
pub(crate) fn is_dispatch_attribute(attr: &syn::Attribute) -> bool {
    ["static_dispatch_for", "extend_static_dispatch"]
        .iter()
        .any(|name| attr.path().is_ident(name))
}

/// An attribute named `static_dispatch` through any path, such as
/// `#[snapfit::static_dispatch]`, `#[di::static_dispatch]` or, imported,
/// `#[static_dispatch]`; one imported under another name cannot be told
/// from any other attribute.
pub(crate) fn is_static_dispatch_attribute(attr: &syn::Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "static_dispatch")
}

/// `#[container]`, with or without settings.
pub(crate) fn is_container_attribute(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("container")
}

/// `#[cfg(...)]`, or a `#[cfg_attr(...)]` that can write one; a `cfg_attr`
/// that writes only other attributes, such as `doc`, leaves the item in
/// every build.
pub(crate) fn is_conditional(attr: &syn::Attribute) -> bool {
    conditional_part(&attr.meta).is_some()
}

/// What of an attribute decides whether its item exists, which a forwarding
/// item keeps so that it exists exactly when the trait's own item does: a
/// `cfg` whole, and a `cfg_attr` with only the `cfg` and `cfg_attr` it
/// writes, since another attribute, such as `deprecated`, may not be written
/// on an item of an impl. A `cfg` that a `macro_rules!` passes as `$m:meta`
/// is read through the group the macro wraps around it.
pub(crate) fn conditional_part(meta: &syn::Meta) -> Option<syn::Meta> {
    if meta.path().is_ident("cfg") {
        return Some(meta.clone());
    }
    let syn::Meta::List(list) = meta else {
        return None;
    };
    if !list.path.is_ident("cfg_attr") {
        return None;
    }
    let Ok((condition, written)) = list.parse_args_with(cfg_attr_arguments) else {
        // rustc reports a cfg_attr it cannot read; until then, it may remove the item.
        return Some(meta.clone());
    };
    let kept: Vec<_> = written.iter().filter_map(conditional_part).collect();
    if kept.is_empty() {
        return None;
    }
    Some(parse_quote!(cfg_attr(#condition, #(#kept),*)))
}

/// A `cfg_attr`'s condition, which may be `true` or `false` as well as a
/// predicate, then the attributes it writes.
fn cfg_attr_arguments(input: ParseStream) -> syn::Result<(TokenStream, Vec<syn::Meta>)> {
    let condition = if input.peek(syn::LitBool) {
        input.parse::<syn::LitBool>()?.into_token_stream()
    } else {
        input.parse::<syn::Meta>()?.into_token_stream()
    };
    input.parse::<Token![,]>()?;
    let written = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;
    Ok((condition, written.into_iter().collect()))
}

#[cfg(test)]
mod tests;

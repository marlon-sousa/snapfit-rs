//! Adapter: the alias of each type the container dispatches statically, and
//! the checks that it and each of its implementations implement its trait,
//! and that each other spelling of an implementation names the same type.

mod dispatches;
#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;

use super::spanned::spanned_runtime;
use crate::adapters::syntax::{DispatchSyntax, DispatchedTypeFallbackSyntax, DispatchedTypeSyntax};
use crate::domain::{DispatchedTypeFallback, Respelling, Variant};
use dispatches::dispatches;

/// Each alias with its checks.
pub(super) fn emit_dispatched_types(
    types: &[DispatchedTypeSyntax],
    vis: &syn::Visibility,
    runtime: &syn::Path,
) -> TokenStream {
    types
        .iter()
        .map(|planned| emit_checked(planned, vis, runtime))
        .collect()
}

/// Each alias alone, for a container that cannot be planned, so that its
/// struct, kept as written, still names the types: `()` stands for a type
/// that holds no implementation, which is refused on its own.
pub(crate) fn emit_fallback_aliases(
    types: &[DispatchedTypeFallbackSyntax],
    vis: &syn::Visibility,
    runtime: &syn::Path,
) -> TokenStream {
    types
        .iter()
        .map(|fallback| emit_fallback_alias(fallback, vis, runtime))
        .collect()
}

fn emit_fallback_alias(
    fallback: &DispatchedTypeFallbackSyntax,
    vis: &syn::Visibility,
    runtime: &syn::Path,
) -> TokenStream {
    match fallback {
        DispatchedTypeFallback::Planned(planned) => emit_alias(planned, vis, runtime),
        DispatchedTypeFallback::Unplanned(DispatchSyntax { name, .. }) => quote! {
            #vis type #name = ();
        },
    }
}

fn emit_checked(
    planned: &DispatchedTypeSyntax,
    vis: &syn::Visibility,
    runtime: &syn::Path,
) -> TokenStream {
    let alias = emit_alias(planned, vis, runtime);
    let checks = emit_checks(planned, runtime);
    quote!(#alias #checks)
}

/// An implementation more private than the alias is reported at the user's
/// own items, not at the alias snapfit writes.
fn emit_alias(
    planned: &DispatchedTypeSyntax,
    vis: &syn::Visibility,
    runtime: &syn::Path,
) -> TokenStream {
    let name = &planned.syntax.name;
    let doc = alias_doc(&planned.syntax);
    let nested = nested(&planned.first, &planned.rest, runtime);
    quote! {
        #[doc = #doc]
        #[allow(private_interfaces)]
        #vis type #name = #nested;
    }
}

/// The trait by its last segment, as a reader of the container's module
/// knows it.
pub(super) fn trait_name(trait_path: &syn::Path) -> String {
    trait_path
        .segments
        .last()
        .map(|segment| segment.ident.unraw())
        .map_or_else(String::new, |ident| ident.to_string())
}

fn alias_doc(DispatchSyntax { trait_path, name }: &DispatchSyntax) -> String {
    let trait_name = trait_name(trait_path);
    format!(
        "One of the implementations of `{trait_name}` that the container dispatches statically; `{}::of(value)` wraps one.",
        name.unraw()
    )
}

/// `Either<First, Either<Second, Only<Last>>>`.
fn nested(
    first: &Variant<syn::Type>,
    rest: &[Variant<syn::Type>],
    runtime: &syn::Path,
) -> TokenStream {
    let ty = &first.syntax;
    match rest.split_first() {
        None => quote!(#runtime::__private::Only<#ty>),
        Some((next, more)) => {
            let rest = nested(next, more, runtime);
            quote!(#runtime::__private::Either<#ty, #rest>)
        }
    }
}

/// The dispatched type implements the trait, which snapfit's words report
/// at the trait in `static_dispatch_for`, and so does each implementation,
/// which rustc reports at the type where a field or a list names it. Each other spelling
/// of an implementation must be the type it holds; when it is not, snapfit's
/// own message reports it there.
fn emit_checks(planned: &DispatchedTypeSyntax, runtime: &syn::Path) -> TokenStream {
    let DispatchSyntax { trait_path, name } = &planned.syntax;
    let dispatched = format_ident!("{}", name, span = trait_path.span());
    let implementations = std::iter::once(&planned.first)
        .chain(&planned.rest)
        .map(|variant| &variant.syntax)
        .map(implements);
    let respellings = planned
        .respelled
        .iter()
        .map(|respelling| same_type(respelling, runtime));
    let dispatches = dispatches(&planned.syntax, &dispatched);
    quote! {
        const _: () = {
            #dispatches
            fn __snapfit_implements<__SnapfitDispatched: ?::core::marker::Sized + #trait_path>() {}
            #(#implementations)*
            #(#respellings)*
        };
    }
}

/// The check of one implementation, spanned at the type that names it.
fn implements(ty: &syn::Type) -> TokenStream {
    quote_spanned!(ty.span()=> let _ = __snapfit_implements::<#ty>;)
}

/// The check of one other spelling, spanned at it.
fn same_type(respelling: &Respelling<syn::Type>, runtime: &syn::Path) -> TokenStream {
    let Respelling { kept, written } = respelling;
    let runtime = spanned_runtime(runtime, written.span());
    quote_spanned!(written.span()=> let _ = #runtime::__private::same_type::<#written, #kept>;)
}

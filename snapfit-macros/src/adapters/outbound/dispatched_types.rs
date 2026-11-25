//! Adapter: the alias of each type the container dispatches statically, and
//! the checks that it and each of its implementations implement its trait.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use crate::adapters::syntax::{DispatchSyntax, DispatchedTypeSyntax};
use crate::domain::Variant;

#[cfg(test)]
mod tests;

/// Each alias with its checks.
pub(super) fn emit_dispatched_types(
    types: &[DispatchedTypeSyntax],
    vis: &syn::Visibility,
) -> TokenStream {
    types
        .iter()
        .map(|planned| emit_checked(planned, vis))
        .collect()
}

/// Each alias alone, for a container that cannot be planned, so that its
/// struct, kept as written, still names the types.
pub(crate) fn emit_aliases(types: &[DispatchedTypeSyntax], vis: &syn::Visibility) -> TokenStream {
    types
        .iter()
        .map(|planned| emit_alias(planned, vis))
        .collect()
}

fn emit_checked(planned: &DispatchedTypeSyntax, vis: &syn::Visibility) -> TokenStream {
    let alias = emit_alias(planned, vis);
    let checks = emit_checks(planned);
    quote!(#alias #checks)
}

/// An implementation more private than the alias is reported at the user's
/// own items, not at the alias snapfit writes.
fn emit_alias(planned: &DispatchedTypeSyntax, vis: &syn::Visibility) -> TokenStream {
    let name = &planned.syntax.name;
    let nested = nested(&planned.first, &planned.rest);
    quote! {
        #[allow(private_interfaces)]
        #vis type #name = #nested;
    }
}

/// `Either<First, Either<Second, Only<Last>>>`.
fn nested(first: &Variant<syn::Type>, rest: &[Variant<syn::Type>]) -> TokenStream {
    let ty = &first.syntax;
    match rest.split_first() {
        None => quote!(::snapfit::__private::Only<#ty>),
        Some((next, more)) => {
            let rest = nested(next, more);
            quote!(::snapfit::__private::Either<#ty, #rest>)
        }
    }
}

/// The dispatched type implements the trait, which rustc reports at the
/// trait in `static_dispatch_for`, and so does each implementation, which it
/// reports at the type where a field or a list names it.
fn emit_checks(planned: &DispatchedTypeSyntax) -> TokenStream {
    let DispatchSyntax { trait_path, name } = &planned.syntax;
    let dispatched = format_ident!("{}", name, span = trait_path.span());
    let implementations = std::iter::once(&planned.first)
        .chain(&planned.rest)
        .map(|variant| &variant.syntax)
        .map(|ty| quote_spanned!(ty.span()=> let _ = __snapfit_implements::<#ty>;));
    quote! {
        const _: () = {
            fn __snapfit_implements<__SnapfitDispatched: ?::core::marker::Sized + #trait_path>() {}
            let _ = __snapfit_implements::<#dispatched>;
            #(#implementations)*
        };
    }
}

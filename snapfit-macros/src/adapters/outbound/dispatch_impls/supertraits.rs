//! Adapter: the check, in words, that snapfit's dispatch types implement
//! each supertrait of a dispatched trait. Plays the role of its parent.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::parse_quote;
use syn::spanned::Spanned;

use super::trait_path::TraitPath;
use crate::adapters::outbound::spanned::spanned_runtime;
use crate::adapters::syntax::{TraitSyntax, is_bare_self, unwrapped};

#[cfg(test)]
mod tests;

pub(super) fn emit_supertrait_checks(syntax: &TraitSyntax) -> TokenStream {
    syntax
        .supertraits
        .iter()
        .enumerate()
        .map(|(position, bound)| check(syntax, position, bound))
        .collect()
}

/// The check that `Only<L>` implements the supertrait whenever `L` implements
/// the trait, reported in words at the supertrait instead of at the impl. Its
/// local trait is named after the trait, the supertrait and its position, so
/// that a type that fails it has no trait of the same name, from the check of
/// another supertrait, for rustc to point at.
fn check(syntax: &TraitSyntax, position: usize, bound: &syn::TraitBound) -> TokenStream {
    let trait_name = syntax.ident.unraw();
    let supertrait = bound
        .path
        .segments
        .last()
        .map_or_else(String::new, |segment| segment.ident.unraw().to_string());
    let message = format!(
        "snapfit cannot dispatch the supertrait `{supertrait}` of `{trait_name}` statically, because it is not marked #[snapfit::static_dispatch] or it is a trait of another crate"
    );
    let label = format!("a supertrait of `{trait_name}` that its dispatch types do not implement");
    let note = format!(
        "mark `{supertrait}` with #[snapfit::static_dispatch] where it is defined; a supertrait of another crate, other than Send, Sync, Sized, Clone and Debug, cannot be dispatched statically, so dispatch `{trait_name}` dynamically instead, through Arc<dyn {trait_name}>"
    );
    let holds = format_ident!(
        "__Snapfit{trait_name}Supertrait{position}{supertrait}",
        span = Span::call_site()
    );
    let path = TraitPath::of(syntax);
    let plain = path.plain();
    let runtime = spanned_runtime(&path.runtime, bound.span());
    let generics = without_bounds_on_self(&syntax.generics);
    let (trait_generics, arguments, where_clause) = generics.split_for_impl();
    let mut holder = generics.clone();
    holder
        .params
        .push(parse_quote!(__SnapfitHolder: ?::core::marker::Sized + #bound));
    let (holder_generics, _, _) = holder.split_for_impl();
    let mut checked = generics.clone();
    checked.params.push(parse_quote!(__SnapfitLast: #plain));
    let (checked_generics, _, _) = checked.split_for_impl();
    let call = quote_spanned!(bound.span()=>
        <#runtime::__private::Only<__SnapfitLast> as #holds #arguments>::__snapfit_holds();
    );
    quote! {
        const _: () = {
            #[diagnostic::on_unimplemented(message = #message, label = #label, note = #note)]
            #[allow(non_camel_case_types)]
            trait #holds #trait_generics #where_clause {
                fn __snapfit_holds() {}
            }
            #[diagnostic::do_not_recommend]
            impl #holder_generics #holds #arguments for __SnapfitHolder #where_clause {}
            #[allow(dead_code)]
            fn __snapfit_supertrait #checked_generics () #where_clause {
                #call
            }
        };
    }
}

/// The trait's generics without its bounds on `Self`, such as
/// `where Self: Clone`, which the check's items have no `Self` to name.
fn without_bounds_on_self(generics: &syn::Generics) -> syn::Generics {
    let mut kept = generics.clone();
    if let Some(clause) = &mut kept.where_clause {
        clause.predicates = clause
            .predicates
            .iter()
            .filter(does_not_bound_self)
            .cloned()
            .collect();
    }
    kept
}

fn does_not_bound_self(predicate: &&syn::WherePredicate) -> bool {
    !matches!(predicate, syn::WherePredicate::Type(predicate) if is_bare_self(unwrapped(&predicate.bounded_ty)))
}

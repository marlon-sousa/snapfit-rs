//! Adapter: the check, in words, that snapfit's dispatch types implement
//! each supertrait of a dispatched trait. Plays the role of its parent.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::parse_quote;
use syn::spanned::Spanned;

use super::trait_path::TraitPath;
use crate::adapters::syntax::TraitSyntax;

#[cfg(test)]
mod tests;

pub(super) fn emit_supertrait_checks(syntax: &TraitSyntax) -> TokenStream {
    syntax
        .supertraits
        .iter()
        .map(|bound| check(syntax, bound))
        .collect()
}

/// `Only<L>` implements the supertrait whenever `L` implements the trait,
/// as for `Send`, `Clone` or a supertrait that carries the attribute. Any
/// other is reported at the supertrait in words, since rustc's own error
/// would name snapfit's dispatch types; `do_not_recommend` keeps rustc from
/// reporting the bound of the impl instead.
fn check(syntax: &TraitSyntax, bound: &syn::TraitBound) -> TokenStream {
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
    let plain = TraitPath::of(syntax).plain();
    let (trait_generics, arguments, where_clause) = syntax.generics.split_for_impl();
    let mut holder = syntax.generics.clone();
    holder
        .params
        .push(parse_quote!(__SnapfitHolder: ?::core::marker::Sized + #bound));
    let (holder_generics, _, _) = holder.split_for_impl();
    let mut checked = syntax.generics.clone();
    checked.params.push(parse_quote!(__SnapfitLast: #plain));
    let (checked_generics, _, _) = checked.split_for_impl();
    let call = quote_spanned!(bound.span()=>
        <::snapfit::__private::Only<__SnapfitLast> as __SnapfitSupertrait #arguments>::__snapfit_holds();
    );
    quote! {
        const _: () = {
            #[diagnostic::on_unimplemented(message = #message, label = #label, note = #note)]
            trait __SnapfitSupertrait #trait_generics #where_clause {
                fn __snapfit_holds() {}
            }
            #[diagnostic::do_not_recommend]
            impl #holder_generics __SnapfitSupertrait #arguments for __SnapfitHolder #where_clause {}
            #[allow(dead_code)]
            fn __snapfit_supertrait #checked_generics () #where_clause {
                #call
            }
        };
    }
}

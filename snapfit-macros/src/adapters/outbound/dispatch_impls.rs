//! Adapter: writes the impls of a dispatched trait for snapfit's two dispatch
//! types, through quote.

mod forwarding;
mod supertraits;
mod target;
mod trait_path;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::syntax::TraitPlan;
use crate::domain::ForwardedItem;
use forwarding::{Forwarding, forwarded_method};
use supertraits::emit_supertrait_checks;
use target::Target;
use trait_path::TraitPath;

/// The trait's impl for `Either`, which holds the first implementation of a
/// list or the rest of it, and for `Only`, which holds the last, with the
/// check of each supertrait.
pub(crate) fn emit_dispatch_impls(plan: &TraitPlan) -> TokenStream {
    let either = emit_impl(plan, Target::Either);
    let only = emit_impl(plan, Target::Only);
    let supertraits = emit_supertrait_checks(&plan.syntax);
    quote! {
        #either
        #only
        #supertraits
    }
}

fn emit_impl(plan: &TraitPlan, target: Target) -> TokenStream {
    let path = TraitPath::of(&plan.syntax);
    let associated: Vec<_> = plan.items.iter().filter_map(associated_type).collect();
    let mut generics = plan.syntax.generics.clone();
    generics.params.extend(target.parameters());
    let (impl_generics, _, _) = generics.split_for_impl();
    let predicates = plan
        .syntax
        .generics
        .where_clause
        .iter()
        .flat_map(|clause| &clause.predicates);
    let bounds = target.bounds(&path, &associated);
    let self_type = target.self_type(&path.runtime);
    // Each supertrait is assumed, so that one the dispatch type does not
    // implement fails where the container uses it, and in words where
    // `emit_supertrait_checks` checks it, not at this impl.
    let supertraits = plan.syntax.supertraits.iter();
    let items = plan
        .items
        .iter()
        .map(|item| forwarded_item(item, target, &path));
    let trait_path = path.plain();
    quote! {
        #[automatically_derived]
        #[doc(hidden)]
        #[allow(deprecated)]
        impl #impl_generics #trait_path for #self_type
        where
            #(#predicates,)*
            #(#bounds,)*
            #(#self_type: #supertraits,)*
        {
            #(#items)*
        }
    }
}

/// The forwarding item of `item`; a deprecated item is still forwarded,
/// which is why the impl allows `deprecated`.
fn forwarded_item(
    item: &ForwardedItem<syn::TraitItemFn, syn::TraitItemType>,
    target: Target,
    path: &TraitPath,
) -> TokenStream {
    match item {
        ForwardedItem::Method {
            syntax,
            rebuilding,
            execution,
            safety,
        } => {
            let forwarding = Forwarding {
                rebuilding: *rebuilding,
                execution: *execution,
                safety: *safety,
            };
            forwarded_method(syntax, forwarding, target, path)
        }
        ForwardedItem::AssociatedType(syntax) => target.associated_type(syntax, path),
    }
}

fn associated_type<M>(item: &ForwardedItem<M, syn::TraitItemType>) -> Option<&syn::Ident> {
    match item {
        ForwardedItem::AssociatedType(syntax) => Some(&syntax.ident),
        ForwardedItem::Method { .. } => None,
    }
}

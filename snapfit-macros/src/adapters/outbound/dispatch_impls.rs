//! Adapter: writes the impls of a dispatched trait for snapfit's two dispatch
//! types, through quote.

mod forwarding;
mod target;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::syntax::TraitPlan;
use crate::domain::ForwardedItem;
use forwarding::forwarded_method;
use target::{Target, TraitPath};

/// The trait's impl for `Either`, which holds the first implementation of a
/// list or the rest of it, and for `Only`, which holds the last.
pub(crate) fn emit_dispatch_impls(plan: &TraitPlan) -> TokenStream {
    let either = emit_impl(plan, Target::Either);
    let only = emit_impl(plan, Target::Only);
    quote! {
        #either
        #only
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
    let self_type = target.self_type();
    let items = plan.items.iter().map(|item| match item {
        ForwardedItem::Method { syntax, rebuilding } => {
            forwarded_method(syntax, *rebuilding, target, &path)
        }
        ForwardedItem::AssociatedType(syntax) => target.associated_type(syntax, &path),
    });
    let trait_path = path.plain();
    quote! {
        #[automatically_derived]
        impl #impl_generics #trait_path for #self_type
        where
            #(#predicates,)*
            #(#bounds,)*
        {
            #(#items)*
        }
    }
}

fn associated_type<M>(item: &ForwardedItem<M, syn::TraitItemType>) -> Option<&syn::Ident> {
    match item {
        ForwardedItem::AssociatedType(syntax) => Some(&syntax.ident),
        ForwardedItem::Method { .. } => None,
    }
}

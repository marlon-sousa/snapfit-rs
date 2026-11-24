//! SPIKE: `#[dispatchable]` on a trait implements the trait for `OneOf2` and
//! `OneOf3`, in the trait's own module, so every type its methods name is in
//! scope. Each method matches on the variant and forwards to it.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as Tokens};
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemTrait, Pat, TraitItem};

const VARIANTS: [&str; 3] = ["First", "Second", "Third"];

#[proc_macro_attribute]
pub fn dispatchable(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item_trait = parse_macro_input!(item as ItemTrait);
    let impls = (2..=3).map(|arity| impl_for(&item_trait, arity));
    quote!(#item_trait #(#impls)*).into()
}

fn impl_for(item_trait: &ItemTrait, arity: usize) -> Tokens {
    let trait_ident = &item_trait.ident;
    let params: Vec<_> = (0..arity).map(|i| format_ident!("__V{}", i)).collect();
    let enum_path = format_ident!("OneOf{}", arity);
    let variants: Vec<_> = VARIANTS[..arity].iter().map(|v| syn::Ident::new(v, Span::call_site())).collect();
    let (_, trait_ty_generics, _) = item_trait.generics.split_for_impl();
    let trait_params = &item_trait.generics.params;
    let first = &params[0];
    // Associated types: every variant must agree with the first one.
    let assoc: Vec<_> = item_trait.items.iter().filter_map(|i| match i { TraitItem::Type(t) => Some(&t.ident), _ => None }).collect();
    let bounds = params.iter().enumerate().map(|(i, p)| {
        if i == 0 || assoc.is_empty() {
            quote!(#p: #trait_ident #trait_ty_generics)
        } else {
            let eqs = assoc.iter().map(|a| quote!(#a = <#first as #trait_ident #trait_ty_generics>::#a));
            quote!(#p: #trait_ident<#(#eqs),*>)
        }
    });
    let assoc_items = assoc.iter().map(|a| quote!(type #a = <#first as #trait_ident #trait_ty_generics>::#a;));
    let methods = item_trait.items.iter().filter_map(|i| match i { TraitItem::Fn(f) => Some(f), _ => None }).map(|f| {
        let mut sig = f.sig.clone();
        let mut args = Vec::new();
        for (n, input) in sig.inputs.iter_mut().enumerate() {
            if let FnArg::Typed(pat_ty) = input {
                // Keep the argument's own name, so that an editor shows it.
                let name = match &*pat_ty.pat {
                    Pat::Ident(ident) => ident.ident.clone(),
                    _ => format_ident!("__arg{}", n),
                };
                *pat_ty.pat = Pat::Ident(syn::PatIdent { attrs: vec![], by_ref: None, mutability: None, ident: name.clone(), subpat: None });
                args.push(name);
            }
        }
        let method = &sig.ident;
        let dot_await = sig.asyncness.map(|_| quote!(.await));
        let arms = variants.iter().zip(&params).map(|(v, p)| quote!(::oneof::#enum_path::#v(__inner) => <#p as #trait_ident #trait_ty_generics>::#method(__inner, #(#args),*) #dot_await,));
        quote!(#sig { match self { #(#arms)* } })
    });
    let all_params = trait_params
        .iter()
        .map(|p| quote!(#p))
        .chain(params.iter().map(|p| quote!(#p)));
    quote! {
        impl<#(#all_params),*> #trait_ident #trait_ty_generics for ::oneof::#enum_path<#(#params),*>
        where #(#bounds),*
        {
            #(#assoc_items)*
            #(#methods)*
        }
    }
}

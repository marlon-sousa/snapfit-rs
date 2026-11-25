//! Adapter: the trait as the impls of a dispatched trait name it. Plays the
//! role of its parent.

use proc_macro2::TokenStream;
use quote::quote;

use crate::adapters::syntax::TraitSyntax;

/// The trait as the impls name it, with its own parameters as arguments.
pub(super) struct TraitPath {
    ident: syn::Ident,
    arguments: Vec<TokenStream>,
}

impl TraitPath {
    pub(super) fn of(syntax: &TraitSyntax) -> Self {
        Self {
            ident: syntax.ident.clone(),
            arguments: syntax.generics.params.iter().map(argument).collect(),
        }
    }

    /// `Store<T>`, or `Database` for a trait with no parameters.
    pub(super) fn plain(&self) -> TokenStream {
        self.with(&[])
    }

    /// `Store<T, Row = <F as Store<T>>::Row>`: the trait with each associated
    /// type equal to `leading`'s.
    pub(super) fn agreeing_with(
        &self,
        leading: &syn::Ident,
        associated: &[&syn::Ident],
    ) -> TokenStream {
        let plain = self.plain();
        let bindings: Vec<_> = associated
            .iter()
            .map(|name| binding(name, leading, &plain))
            .collect();
        self.with(&bindings)
    }

    fn with(&self, bindings: &[TokenStream]) -> TokenStream {
        let ident = &self.ident;
        let arguments = self.arguments.iter().chain(bindings);
        if self.arguments.is_empty() && bindings.is_empty() {
            quote!(#ident)
        } else {
            quote!(#ident<#(#arguments),*>)
        }
    }
}

fn binding(name: &syn::Ident, leading: &syn::Ident, plain: &TokenStream) -> TokenStream {
    quote!(#name = <#leading as #plain>::#name)
}

fn argument(param: &syn::GenericParam) -> TokenStream {
    match param {
        syn::GenericParam::Lifetime(param) => {
            let lifetime = &param.lifetime;
            quote!(#lifetime)
        }
        syn::GenericParam::Type(param) => {
            let ident = &param.ident;
            quote!(#ident)
        }
        syn::GenericParam::Const(param) => {
            let ident = &param.ident;
            quote!(#ident)
        }
    }
}

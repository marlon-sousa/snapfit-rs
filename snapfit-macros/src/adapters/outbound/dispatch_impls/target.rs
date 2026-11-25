//! Adapter: what differs between the impls for `Either` and for `Only`, the
//! parameters, bounds and variants each is written with. Plays the role of
//! its parent.

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::adapters::syntax::TraitSyntax;

/// Which of snapfit's dispatch types an impl is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    Either,
    Only,
}

/// One variant of a dispatch type, as a `match` arm reads and rebuilds it.
pub(super) struct Variant {
    /// The type parameter of the implementation the variant holds.
    pub(super) implementation: syn::Ident,
    /// The variant's constructor, such as `::snapfit::__private::Either::First`.
    pub(super) constructor: TokenStream,
}

impl Target {
    pub(super) fn parameters(self) -> impl Iterator<Item = syn::GenericParam> {
        self.variants().into_iter().map(type_parameter)
    }

    pub(super) fn self_type(self) -> TokenStream {
        match self {
            Self::Either => {
                let (first, rest) = (first(), rest());
                quote!(::snapfit::__private::Either<#first, #rest>)
            }
            Self::Only => {
                let last = last();
                quote!(::snapfit::__private::Only<#last>)
            }
        }
    }

    pub(super) fn variants(self) -> Vec<Variant> {
        match self {
            Self::Either => vec![
                Variant {
                    implementation: first(),
                    constructor: quote!(::snapfit::__private::Either::First),
                },
                Variant {
                    implementation: rest(),
                    constructor: quote!(::snapfit::__private::Either::Rest),
                },
            ],
            Self::Only => vec![Variant {
                implementation: last(),
                constructor: quote!(::snapfit::__private::Only),
            }],
        }
    }

    /// The implementation whose associated types the dispatch type takes.
    fn leading(self) -> syn::Ident {
        match self {
            Self::Either => first(),
            Self::Only => last(),
        }
    }

    /// Every variant implements the trait, the rest of a list with the same
    /// associated types as its first implementation.
    pub(super) fn bounds(self, path: &TraitPath, associated: &[&syn::Ident]) -> Vec<TokenStream> {
        let plain = path.plain();
        let leading = self.leading();
        match self {
            Self::Either => {
                let rest = rest();
                let agreeing = path.agreeing_with(&leading, associated);
                vec![quote!(#leading: #plain), quote!(#rest: #agreeing)]
            }
            Self::Only => vec![quote!(#leading: #plain)],
        }
    }

    pub(super) fn associated_type(
        self,
        syntax: &syn::TraitItemType,
        path: &TraitPath,
    ) -> TokenStream {
        let ident = &syntax.ident;
        let leading = self.leading();
        let plain = path.plain();
        let cfgs = syntax.attrs.iter().filter(is_cfg);
        quote! {
            #(#cfgs)*
            type #ident = <#leading as #plain>::#ident;
        }
    }
}

fn type_parameter(variant: Variant) -> syn::GenericParam {
    syn::GenericParam::Type(variant.implementation.into())
}

/// `#[cfg(...)]`, which the impl's item keeps so that it exists exactly
/// when the trait's does.
pub(super) fn is_cfg(attr: &&syn::Attribute) -> bool {
    attr.path().is_ident("cfg")
}

fn first() -> syn::Ident {
    syn::Ident::new("__SnapfitFirst", Span::call_site())
}

fn rest() -> syn::Ident {
    syn::Ident::new("__SnapfitRest", Span::call_site())
}

fn last() -> syn::Ident {
    syn::Ident::new("__SnapfitLast", Span::call_site())
}

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
    fn agreeing_with(&self, leading: &syn::Ident, associated: &[&syn::Ident]) -> TokenStream {
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

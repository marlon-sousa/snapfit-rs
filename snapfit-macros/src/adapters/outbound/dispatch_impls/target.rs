//! Adapter: what differs between the impls for `Either` and for `Only`, the
//! parameters, bounds, variants and associated types each is written with.
//! Plays the role of its parent.

use proc_macro2::{Span, TokenStream};
use quote::quote;

use super::trait_path::TraitPath;
use crate::adapters::syntax::is_cfg;

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

fn first() -> syn::Ident {
    syn::Ident::new("__SnapfitFirst", Span::call_site())
}

fn rest() -> syn::Ident {
    syn::Ident::new("__SnapfitRest", Span::call_site())
}

fn last() -> syn::Ident {
    syn::Ident::new("__SnapfitLast", Span::call_site())
}

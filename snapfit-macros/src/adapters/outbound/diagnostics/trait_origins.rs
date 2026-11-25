//! Adapter: the compiler errors for every problem of a dispatched trait,
//! each underlining the item, the `unsafe` or the arguments it is about.
//! Plays the role of its parent.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use crate::domain::{DispatchSubject, DomainError, DomainErrors, ItemPosition};

/// The tokens a diagnostic about a dispatched trait underlines.
#[derive(Debug, Clone)]
pub(crate) struct TraitOrigins {
    /// The trait's name, or the whole item when it is not a trait.
    annotated: TokenStream,
    /// The attribute's arguments, or the annotated item when it has none.
    arguments: TokenStream,
    safety: Option<Span>,
    /// Every item of the trait, in order, so that an item's position finds it.
    items: Vec<TokenStream>,
}

impl TraitOrigins {
    pub(crate) fn new(args: &TokenStream, item: &syn::Item) -> Self {
        let (annotated, safety, items) = match item {
            syn::Item::Trait(item) => (
                item.ident.to_token_stream(),
                item.unsafety.map(|unsafety| unsafety.span),
                item.items.iter().map(item_tokens).collect(),
            ),
            _ => (item.to_token_stream(), None, Vec::new()),
        };
        let arguments = if args.is_empty() {
            annotated.clone()
        } else {
            args.clone()
        };
        Self {
            annotated,
            arguments,
            safety,
            items,
        }
    }

    /// One `compile_error!` per problem, in the order they were found.
    pub(crate) fn compile_errors(&self, errors: &DomainErrors) -> TokenStream {
        errors
            .iter()
            .map(|error| self.error_for(error).to_compile_error())
            .collect()
    }

    fn error_for(&self, error: &DomainError) -> syn::Error {
        match error.dispatch_subject() {
            DispatchSubject::Item(position) => {
                syn::Error::new_spanned(self.item_at(position), error)
            }
            DispatchSubject::Arguments => syn::Error::new_spanned(&self.arguments, error),
            DispatchSubject::Safety => match self.safety {
                Some(span) => syn::Error::new(span, error),
                None => syn::Error::new_spanned(&self.annotated, error),
            },
            DispatchSubject::Annotated => syn::Error::new_spanned(&self.annotated, error),
        }
    }

    fn item_at(&self, position: ItemPosition) -> &TokenStream {
        self.items.get(position.0).unwrap_or(&self.annotated)
    }
}

/// A method is underlined at its signature, so that a long default body
/// does not take the diagnostic away from the line that is wrong.
fn item_tokens(item: &syn::TraitItem) -> TokenStream {
    match item {
        syn::TraitItem::Fn(method) => method.sig.to_token_stream(),
        _ => item.to_token_stream(),
    }
}

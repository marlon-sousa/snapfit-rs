//! Adapter: the compiler errors for every problem of a dispatched trait,
//! each underlining the item, the `unsafe` or the arguments it is about.
//! Plays the role of its parent.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use crate::adapters::syntax::is_static_dispatch_attribute;
use crate::domain::{DispatchSubject, DomainError, DomainErrors, ItemPosition};

/// The tokens a diagnostic about a dispatched trait underlines.
#[derive(Debug, Clone)]
pub(crate) struct TraitOrigins {
    /// The trait's name, or the whole item when it is not a trait.
    annotated: TokenStream,
    /// The attribute's arguments, or the annotated item when it has none.
    arguments: TokenStream,
    /// The next `#[snapfit::static_dispatch]` on the trait, or its name when
    /// it has none.
    next_mark: TokenStream,
    safety: Option<Span>,
    /// The trait's supertraits, parameters and `where` clause, or its name
    /// when it has none.
    bounds: TokenStream,
    /// Every item of the trait, in order, so that an item's position finds it.
    items: Vec<TokenStream>,
}

impl TraitOrigins {
    pub(crate) fn new(args: &TokenStream, item: &syn::Item) -> Self {
        let (annotated, safety, bounds, next_mark, items) = match item {
            syn::Item::Trait(item) => (
                item.ident.to_token_stream(),
                item.unsafety.map(|unsafety| unsafety.span),
                bounds_tokens(item),
                next_mark_tokens(item),
                item.items.iter().map(item_tokens).collect(),
            ),
            _ => (
                item.to_token_stream(),
                None,
                TokenStream::new(),
                TokenStream::new(),
                Vec::new(),
            ),
        };
        let next_mark = if next_mark.is_empty() {
            annotated.clone()
        } else {
            next_mark
        };
        let bounds = if bounds.is_empty() {
            annotated.clone()
        } else {
            bounds
        };
        let arguments = if args.is_empty() {
            annotated.clone()
        } else {
            args.clone()
        };
        Self {
            annotated,
            arguments,
            next_mark,
            safety,
            bounds,
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
            DispatchSubject::NextMark => syn::Error::new_spanned(&self.next_mark, error),
            DispatchSubject::Safety => match self.safety {
                Some(span) => syn::Error::new(span, error),
                None => syn::Error::new_spanned(&self.annotated, error),
            },
            DispatchSubject::Bounds => syn::Error::new_spanned(&self.bounds, error),
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

/// The first `#[snapfit::static_dispatch]` the trait still carries, which
/// rustc expands after this one.
fn next_mark_tokens(item: &syn::ItemTrait) -> TokenStream {
    item.attrs
        .iter()
        .find(|attr| is_static_dispatch_attribute(attr))
        .to_token_stream()
}

/// `<T: Into<Self>>: Ordered<Self> where ...`, as the trait writes them.
fn bounds_tokens(item: &syn::ItemTrait) -> TokenStream {
    let mut tokens = item.generics.params.to_token_stream();
    item.supertraits.to_tokens(&mut tokens);
    item.generics.where_clause.to_tokens(&mut tokens);
    tokens
}

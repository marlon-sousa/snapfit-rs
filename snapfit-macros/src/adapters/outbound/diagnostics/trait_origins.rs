//! Adapter: the compiler errors for every problem of a dispatched trait,
//! each underlining the item, the `unsafe` or the arguments it is about.
//! Plays the role of its parent.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use crate::domain::{DispatchSubject, DomainError, DomainErrors, TraitItem};

/// The tokens a diagnostic about a dispatched trait underlines.
#[derive(Debug, Clone)]
pub(crate) struct TraitOrigins {
    /// The trait's name, or the whole item when it is not a trait.
    annotated: TokenStream,
    /// The attribute's arguments, or the annotated item when it has none.
    arguments: TokenStream,
    safety: Option<Span>,
    items: Vec<ItemOrigin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    Method,
    AssociatedType,
    AssociatedConst,
    Macro,
}

#[derive(Debug, Clone)]
struct ItemOrigin {
    kind: ItemKind,
    name: String,
    tokens: TokenStream,
}

impl TraitOrigins {
    pub(crate) fn new(args: &TokenStream, item: &syn::Item) -> Self {
        let (annotated, safety, items) = match item {
            syn::Item::Trait(item) => (
                item.ident.to_token_stream(),
                item.unsafety.map(|unsafety| unsafety.span),
                item.items.iter().filter_map(ItemOrigin::of).collect(),
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
        let tokens = match error.dispatch_subject() {
            DispatchSubject::Item(item) => self.item_tokens(item),
            DispatchSubject::Arguments => &self.arguments,
            DispatchSubject::Safety => {
                if let Some(span) = self.safety {
                    return syn::Error::new(span, error);
                }
                &self.annotated
            }
            DispatchSubject::Annotated => &self.annotated,
        };
        syn::Error::new_spanned(tokens, error)
    }

    fn item_tokens(&self, item: &TraitItem) -> &TokenStream {
        let (kind, name) = match item {
            TraitItem::Method(name) => (ItemKind::Method, name.as_str()),
            TraitItem::AssociatedType(name) => (ItemKind::AssociatedType, name.as_str()),
            TraitItem::AssociatedConst(name) => (ItemKind::AssociatedConst, name.as_str()),
            TraitItem::Macro(name) => (ItemKind::Macro, name.as_str()),
        };
        self.items
            .iter()
            .find(|origin| origin.kind == kind && origin.name == name)
            .map_or(&self.annotated, |origin| &origin.tokens)
    }
}

impl ItemOrigin {
    /// A method is underlined at its signature, so that a long default body
    /// does not take the diagnostic away from the line that is wrong.
    fn of(item: &syn::TraitItem) -> Option<Self> {
        let (kind, name, tokens) = match item {
            syn::TraitItem::Fn(method) => (
                ItemKind::Method,
                method.sig.ident.to_string(),
                method.sig.to_token_stream(),
            ),
            syn::TraitItem::Type(associated) => (
                ItemKind::AssociatedType,
                associated.ident.to_string(),
                associated.to_token_stream(),
            ),
            syn::TraitItem::Const(constant) => (
                ItemKind::AssociatedConst,
                constant.ident.to_string(),
                constant.to_token_stream(),
            ),
            syn::TraitItem::Macro(call) => (
                ItemKind::Macro,
                call.mac.path.segments.last()?.ident.to_string(),
                call.to_token_stream(),
            ),
            _ => return None,
        };
        Some(Self { kind, name, tokens })
    }
}

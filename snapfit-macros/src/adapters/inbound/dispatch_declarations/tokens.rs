//! Adapter: the tokens of a dispatch attribute, cut into the parts the
//! reader checks one by one. Plays the role of its parent.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};

use super::super::type_inspector::compact_type_tokens;

/// `<Trait>`, then `as <Name>` when it is written.
pub(super) struct Head {
    pub(super) trait_path: syn::Path,
    pub(super) name: Option<syn::Path>,
}

impl Parse for Head {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let trait_path = input.parse()?;
        let name = match input.parse::<Option<syn::Token![as]>>()? {
            Some(_) => Some(input.parse()?),
            None => None,
        };
        if !input.is_empty() {
            return Err(input.error("expected the word as and the name after the trait"));
        }
        Ok(Self { trait_path, name })
    }
}

/// The parts between the attribute's commas, leaving out empty ones, such
/// as the one a trailing comma leaves.
pub(super) fn chunks(tokens: TokenStream) -> Vec<TokenStream> {
    let mut chunks = vec![TokenStream::new()];
    for tree in tokens {
        match (&tree, chunks.last_mut()) {
            (TokenTree::Punct(punct), _) if punct.as_char() == ',' => {
                chunks.push(TokenStream::new());
            }
            (_, Some(chunk)) => chunk.extend([tree]),
            (_, None) => {}
        }
    }
    chunks.retain(|chunk| !chunk.is_empty());
    chunks
}

/// `<name> = <value>`, as the setting's word and the tokens after `=`.
pub(super) fn setting_name(setting: &TokenStream) -> Option<(syn::Ident, TokenStream)> {
    let mut trees = setting.clone().into_iter();
    let (Some(TokenTree::Ident(name)), Some(TokenTree::Punct(equals))) =
        (trees.next(), trees.next())
    else {
        return None;
    };
    (equals.as_char() == '=').then(|| (name, trees.collect()))
}

/// What square brackets hold, also when a macro passes the list as one
/// value; `None` when the value is anything else.
pub(super) fn bracketed(value: TokenStream) -> Option<TokenStream> {
    let mut trees = value.into_iter();
    let (Some(TokenTree::Group(group)), None) = (trees.next(), trees.next()) else {
        return None;
    };
    match group.delimiter() {
        Delimiter::Bracket => Some(group.stream()),
        Delimiter::None => bracketed(group.stream()),
        Delimiter::Parenthesis | Delimiter::Brace => None,
    }
}

pub(super) fn compact(tokens: TokenStream) -> String {
    compact_type_tokens(tokens)
}

//! Adapter: the tokens of a dispatch attribute, cut into the parts the
//! reader checks one by one. Plays the role of its parent.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};

use derive_new::new;

/// `<Trait>`, then `as <Name>` when it is written, then whatever follows
/// before the first comma, which should be nothing.
pub(super) struct Head {
    pub(super) trait_path: syn::Path,
    pub(super) name: Option<syn::Path>,
    pub(super) rest: TokenStream,
}

impl Parse for Head {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let trait_path = input.parse()?;
        let name = match input.parse::<Option<syn::Token![as]>>()? {
            Some(_) => Some(input.parse()?),
            None => None,
        };
        let rest = input.parse()?;
        Ok(Self {
            trait_path,
            name,
            rest,
        })
    }
}

/// The parts between the attribute's commas, leaving out empty ones, such
/// as the one a trailing comma leaves. A comma between angle brackets, as in
/// `Store<A, B>`, is part of its type.
pub(super) fn chunks(tokens: TokenStream) -> Vec<TokenStream> {
    let mut chunks = tokens.into_iter().fold(Cut::new(), Cut::with).chunks;
    chunks.retain(|chunk| !chunk.is_empty());
    chunks
}

/// The chunks cut so far, how deep in angle brackets the next token is, and
/// whether the token before it was a `-`, whose `>` closes no bracket.
#[derive(new)]
struct Cut {
    #[new(value = "vec![TokenStream::new()]")]
    chunks: Vec<TokenStream>,
    #[new(default)]
    depth: usize,
    #[new(default)]
    after_dash: bool,
}

impl Cut {
    fn with(mut self, tree: TokenTree) -> Self {
        let punct = match &tree {
            TokenTree::Punct(punct) => Some(punct.as_char()),
            _ => None,
        };
        match punct {
            Some(',') if self.depth == 0 => self.chunks.push(TokenStream::new()),
            Some('<') => self.depth += 1,
            Some('>') if !self.after_dash => self.depth = self.depth.saturating_sub(1),
            _ => {}
        }
        if let (false, Some(chunk)) = (
            punct == Some(',') && self.depth == 0,
            self.chunks.last_mut(),
        ) {
            chunk.extend([tree]);
        }
        self.after_dash = punct == Some('-');
        self
    }
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

//! Adapter: the settings of an attribute, as the user wrote them between its
//! parentheses, which both adapters read. Plays the role of its parent.

use proc_macro2::{TokenStream, TokenTree};

/// The settings of an attribute, split at the commas between them, so that
/// one that is no `name = value` a `syn::Meta` reads, such as
/// `remote = other::Store<u32>`, leaves the others readable. A trailing
/// comma ends the last setting; any other comma leaves one, even empty,
/// after it.
pub(crate) fn settings(args: &TokenStream) -> Vec<TokenStream> {
    let tokens: Vec<TokenTree> = args.clone().into_iter().collect();
    let written = match tokens.split_last() {
        None => return Vec::new(),
        Some((last, rest)) if is_comma(last) => rest,
        Some(_) => &tokens,
    };
    written
        .split(is_comma)
        .map(|setting| setting.iter().cloned().collect())
        .collect()
}

fn is_comma(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == ',')
}

#[cfg(test)]
mod tests;

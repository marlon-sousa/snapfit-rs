//! Adapter: the path to snapfit's runtime crate, given the span of the
//! user's code a check stands for, so that rustc's error for the check
//! points there and only there, as `quote_spanned!` does for the tokens it
//! writes itself.

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::ToTokens;

/// `runtime`'s tokens, each at `span`.
pub(in crate::adapters::outbound) fn spanned_runtime(
    runtime: &syn::Path,
    span: Span,
) -> TokenStream {
    respanned(runtime.to_token_stream(), span)
}

fn respanned(tokens: TokenStream, span: Span) -> TokenStream {
    tokens
        .into_iter()
        .map(|token| respanned_token(token, span))
        .collect()
}

fn respanned_token(token: TokenTree, span: Span) -> TokenTree {
    let mut token = match token {
        TokenTree::Group(group) => TokenTree::Group(Group::new(
            group.delimiter(),
            respanned(group.stream(), span),
        )),
        other => other,
    };
    token.set_span(span);
    token
}

#[cfg(test)]
mod tests;

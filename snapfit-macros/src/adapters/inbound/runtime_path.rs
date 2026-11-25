//! Adapter: the path through which the generated code reaches snapfit's
//! runtime crate, `crate = <path>` or `::snapfit`.

use proc_macro2::{TokenStream, TokenTree};
use syn::parse::{ParseStream, Parser};
use syn::{Token, parse_quote};

use super::darling_models::RuntimeMeta;
use crate::domain::{ArgumentProblem, AttributeProblem, DomainError};

/// `::snapfit`, when the attribute gives no `crate`.
pub(in crate::adapters) fn default_runtime_path() -> syn::Path {
    parse_quote!(::snapfit)
}

/// The path `crate = ...` gives, or the container's problem with what it
/// was given.
pub(super) fn runtime_path(meta: Option<RuntimeMeta>) -> Result<syn::Path, DomainError> {
    let problem = match meta {
        None => return Ok(default_runtime_path()),
        Some(RuntimeMeta::Path(path)) => return Ok(path),
        Some(RuntimeMeta::Quoted(text)) => ArgumentProblem::QuotedRuntimePath(text),
        Some(RuntimeMeta::Other(text)) => ArgumentProblem::NotARuntimePath(text),
    };
    Err(AttributeProblem::Container { problem }.into())
}

/// The `crate` path among `args` that can be read, whatever else is wrong
/// with them, so that what a refused container still writes reaches snapfit.
pub(crate) fn readable_runtime_path(args: &TokenStream) -> syn::Path {
    settings(args)
        .iter()
        .find_map(crate_path)
        .unwrap_or_else(default_runtime_path)
}

/// The settings of an attribute, split at the commas between them, so that
/// one that is no `name = value` a `syn::Meta` reads, such as
/// `remote = other::Store<u32>`, leaves the others readable. A trailing
/// comma ends the last setting; any other comma leaves one, even empty,
/// after it.
pub(super) fn settings(args: &TokenStream) -> Vec<TokenStream> {
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

/// The path of a `crate = <path>` setting.
pub(super) fn crate_path(setting: &TokenStream) -> Option<syn::Path> {
    crate_setting.parse2(setting.clone()).ok()
}

fn crate_setting(input: ParseStream) -> syn::Result<syn::Path> {
    input.parse::<Token![crate]>()?;
    input.parse::<Token![=]>()?;
    syn::Path::parse_mod_style(input)
}

#[cfg(test)]
mod tests;

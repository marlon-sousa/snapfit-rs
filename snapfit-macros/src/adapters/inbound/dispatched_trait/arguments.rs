//! Adapter: reads the arguments of `#[snapfit::static_dispatch]`. Plays the
//! role of its parent.

use proc_macro2::{TokenStream, TokenTree};

use super::super::runtime_path::{crate_path, default_runtime_path};
use crate::adapters::syntax::settings;
use crate::domain::DispatchArguments;

/// How many settings of each kind `args` holds, and the path of its first
/// `crate = <path>`, or `::snapfit`.
pub(super) fn arguments(args: &TokenStream) -> (DispatchArguments, syn::Path) {
    let settings = settings(args);
    let paths: Vec<_> = settings.iter().filter_map(crate_path).collect();
    let remote = settings.iter().filter(|setting| is_remote(setting)).count();
    let other = settings.len() - paths.len() - remote;
    let arguments = DispatchArguments::new(remote, paths.len(), other);
    let runtime = paths
        .into_iter()
        .next()
        .unwrap_or_else(default_runtime_path);
    (arguments, runtime)
}

/// `remote = ` and something after it.
fn is_remote(setting: &TokenStream) -> bool {
    let mut tokens = setting.clone().into_iter();
    match (tokens.next(), tokens.next(), tokens.next()) {
        (Some(TokenTree::Ident(name)), Some(TokenTree::Punct(equals)), Some(_)) => {
            name == "remote" && equals.as_char() == '='
        }
        _ => false,
    }
}

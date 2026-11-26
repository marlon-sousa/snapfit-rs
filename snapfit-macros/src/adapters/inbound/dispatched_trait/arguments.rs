//! Adapter: reads the arguments of `#[snapfit::static_dispatch]`. Plays the
//! role of its parent.

use darling::FromMeta;
use proc_macro2::TokenStream;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::{ParseStream, Parser};

use super::super::darling_models::RuntimeMeta;
use super::super::runtime_path::{crate_path, default_runtime_path};
use crate::adapters::syntax::settings;
use crate::domain::{DispatchArguments, DispatchSetting, RuntimeValue};

/// Every setting `args` holds, and the path of its first readable
/// `crate = <path>`, or `::snapfit`.
pub(super) fn arguments(args: &TokenStream) -> (DispatchArguments, syn::Path) {
    let settings = settings(args);
    let runtime = settings
        .iter()
        .find_map(crate_path)
        .unwrap_or_else(default_runtime_path);
    let arguments = DispatchArguments::new(settings.iter().map(setting).collect());
    (arguments, runtime)
}

fn setting(tokens: &TokenStream) -> DispatchSetting {
    match runtime_value.parse2(tokens.clone()) {
        Ok(value) => DispatchSetting::Runtime(value),
        Err(_) if is_remote(tokens) => DispatchSetting::Remote,
        Err(_) => DispatchSetting::Other,
    }
}

/// `crate = ` and a value, read as `#[container]` reads its own `crate`, or
/// `crate` alone.
fn runtime_value(input: ParseStream) -> syn::Result<RuntimeValue> {
    input.parse::<Token![crate]>()?;
    if input.is_empty() {
        return Ok(RuntimeValue::Bare);
    }
    input.parse::<Token![=]>()?;
    let value = input.parse::<syn::Expr>()?;
    Ok(match RuntimeMeta::from_expr(&value)? {
        RuntimeMeta::Path(_) => RuntimeValue::Path,
        RuntimeMeta::Quoted(text) => RuntimeValue::Quoted(text),
        RuntimeMeta::Other(text) => RuntimeValue::NotAPath(text),
        // Never reached: `from_expr` does not return it, and a `crate` alone returns above.
        RuntimeMeta::Bare => RuntimeValue::Bare,
    })
}

/// `remote = ` and something after it, read through the group a `meta`
/// fragment of a `macro_rules!` puts around it.
fn is_remote(setting: &TokenStream) -> bool {
    remote_setting.parse2(setting.clone()).is_ok()
}

fn remote_setting(input: ParseStream) -> syn::Result<()> {
    let name = input.call(syn::Ident::parse_any)?;
    if name != "remote" {
        return Err(syn::Error::new(name.span(), "not remote"));
    }
    input.parse::<Token![=]>()?;
    if input.is_empty() {
        return Err(input.error("no value"));
    }
    input.parse::<TokenStream>().map(drop)
}

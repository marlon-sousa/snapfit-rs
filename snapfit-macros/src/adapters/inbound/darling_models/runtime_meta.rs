//! Adapter: `crate = <path>` on `#[container]`, as darling reads it.

use darling::FromMeta;
use quote::ToTokens;

use super::super::type_inspector::compact_type_tokens;
use super::expressions::ungrouped;

/// The path the generated code reaches snapfit through, or what was
/// written instead, for the adapter to refuse in words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum RuntimeMeta {
    Path(syn::Path),
    /// The string's own text, such as `di`.
    Quoted(String),
    /// The value as written, such as `3`.
    Other(String),
}

impl FromMeta for RuntimeMeta {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        Ok(match ungrouped(expr) {
            syn::Expr::Path(path) if path.qself.is_none() && path.attrs.is_empty() => {
                Self::Path(path.path.clone())
            }
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => Self::Quoted(lit.value()),
            other => Self::Other(compact_type_tokens(other.to_token_stream())),
        })
    }
}

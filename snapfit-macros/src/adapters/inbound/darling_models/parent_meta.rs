//! Adapter: the value of `parent = ...` on `#[container]`, as darling reads it.

use darling::FromMeta;
use quote::quote;

use super::super::type_inspector::compact_type_tokens;
use super::expressions::ungrouped;

/// The parent container's type as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum ParentTypeMeta {
    Named(String),
    /// `parent` with no value.
    Bare,
}

impl FromMeta for ParentTypeMeta {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self::Named(value.to_string()))
    }

    /// A setting with no value, which darling would refuse in its own words.
    fn from_word() -> darling::Result<Self> {
        Ok(Self::Bare)
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        let expr = ungrouped(expr);
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => Self::from_string(&lit.value()),
            syn::Expr::Path(path) => Ok(Self::Named(compact_type_tokens(quote!(#path)))),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
    }
}

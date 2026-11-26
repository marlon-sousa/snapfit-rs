//! Adapter: the value of `parent = ...` on `#[container]`, as darling reads it.

use darling::FromMeta;
use quote::quote;

use super::super::type_inspector::compact_type_tokens;
use super::expressions::ungrouped;

/// The parent container's type as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) struct ParentTypeMeta(pub(in crate::adapters::inbound) String);

impl FromMeta for ParentTypeMeta {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(value.to_string()))
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        let expr = ungrouped(expr);
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => Self::from_string(&lit.value()),
            syn::Expr::Path(path) => Ok(Self(compact_type_tokens(quote!(#path)))),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
    }
}

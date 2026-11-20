//! Adapter: the value of `fallible = ...` as darling reads it.

use darling::FromMeta;
use quote::ToTokens;

use super::expressions::ungrouped;

/// `fallible = true`, with its `fallible` word for its span, `fallible =
/// false`, or the shape it was written in instead, refused when it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum FallibleMeta {
    True(syn::Ident),
    False,
    /// `fallible` with no value.
    Bare,
    /// `fallible(...)`.
    InParentheses,
    /// A value other than `true` or `false`, as written.
    NotABoolean(String),
}

impl FromMeta for FallibleMeta {
    /// Every shape is read here, so that a bare word is not taken as `true`,
    /// as darling takes it for a `bool`.
    fn from_meta(meta: &syn::Meta) -> darling::Result<Self> {
        Ok(match meta {
            syn::Meta::Path(_) => Self::Bare,
            syn::Meta::List(_) => Self::InParentheses,
            syn::Meta::NameValue(pair) => Self::from_value(&pair.path, ungrouped(&pair.value)),
        })
    }
}

impl FallibleMeta {
    fn from_value(path: &syn::Path, value: &syn::Expr) -> Self {
        match (path.get_ident(), value) {
            (
                Some(word),
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Bool(boolean),
                    ..
                }),
            ) if boolean.value => Self::True(word.clone()),
            (
                _,
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Bool(syn::LitBool { value: false, .. }),
                    ..
                }),
            ) => Self::False,
            _ => Self::NotABoolean(value.to_token_stream().to_string()),
        }
    }
}

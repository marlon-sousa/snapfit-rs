//! Adapter: the attribute arguments as darling reads them, before they are checked.

use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;

use super::type_inspector::compact_type_tokens;
use crate::domain::AttributeProblem;

mod build_meta;
mod prerequisites;
#[cfg(test)]
mod tests;

pub(super) use build_meta::{BuildMeta, BuildWord, FunctionNameMeta, single_word};
pub(super) use prerequisites::{PrerequisiteList, PrerequisiteMeta};

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct ContainerAttrMeta {
    #[darling(default)]
    /// `"sync"` or `"async"`, checked by the adapter so that its error names
    /// the mode without darling's location.
    pub(super) mode: Option<String>,
    #[darling(default)]
    pub(super) parent: Option<ParentTypeMeta>,
    #[darling(default)]
    pub(super) introspection: bool,
    #[darling(default)]
    pub(super) leaf_build: Option<BuildMeta>,
}

impl ContainerAttrMeta {
    pub(super) fn from_tokens(tokens: TokenStream) -> Result<Self, AttributeProblem> {
        if tokens.is_empty() {
            return Ok(Self::default());
        }
        let nested = NestedMeta::parse_meta_list(tokens).map_err(unreadable)?;
        Self::from_list(&nested).map_err(unreadable)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParentTypeMeta(pub(super) String);

impl FromMeta for ParentTypeMeta {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(value.to_string()))
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
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

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct DependencyAttrMeta {
    #[darling(default)]
    pub(super) deps: PrerequisiteList,
    #[darling(default)]
    pub(super) is_async: bool,
    #[darling(default)]
    pub(super) build: Option<BuildMeta>,
}

impl DependencyAttrMeta {
    pub(super) fn from_attribute(attr: &syn::Attribute) -> Result<Self, AttributeProblem> {
        match &attr.meta {
            syn::Meta::Path(_) => Ok(Self::default()),
            syn::Meta::List(list) => Self::from_tokens(list.tokens.clone()),
            syn::Meta::NameValue(_) => Err(AttributeProblem::Assigned),
        }
    }

    fn from_tokens(tokens: TokenStream) -> Result<Self, AttributeProblem> {
        let normalized = normalize_async_keyword(tokens);
        let nested = NestedMeta::parse_meta_list(normalized).map_err(unreadable)?;
        Self::from_list(&nested).map_err(unreadable)
    }
}

/// `async` is a keyword, which darling cannot read as a field name.
fn normalize_async_keyword(tokens: TokenStream) -> TokenStream {
    tokens.into_iter().map(async_as_is_async).collect()
}

fn async_as_is_async(tree: TokenTree) -> TokenTree {
    match tree {
        TokenTree::Ident(ident) if ident == "async" => {
            TokenTree::Ident(Ident::new("is_async", ident.span()))
        }
        tree => tree,
    }
}

fn unreadable(err: impl std::fmt::Display) -> AttributeProblem {
    AttributeProblem::Unreadable(err.to_string())
}

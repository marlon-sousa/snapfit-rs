use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;

use super::type_inspector::compact_type_tokens;
use crate::domain::DomainError;

#[cfg(test)]
mod tests;

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct ContainerAttrMeta {
    #[darling(default)]
    pub mode: Option<String>,
    #[darling(default)]
    pub parent: Option<ParentTypeMeta>,
    #[darling(default)]
    pub introspection: bool,
}

impl ContainerAttrMeta {
    pub(super) fn from_tokens(tokens: TokenStream) -> Result<Self, DomainError> {
        if tokens.is_empty() {
            return Ok(Self::default());
        }
        let nested = NestedMeta::parse_meta_list(tokens).map_err(invalid_config)?;
        Self::from_list(&nested).map_err(invalid_config)
    }

    pub(super) fn from_attribute(attr: &syn::Attribute) -> Result<Self, DomainError> {
        match &attr.meta {
            syn::Meta::Path(_) => Ok(Self::default()),
            syn::Meta::List(list) => Self::from_tokens(list.tokens.clone()),
            syn::Meta::NameValue(_) => Err(DomainError::InvalidConfiguration(
                "#[container] attribute does not accept name-value assignment".to_string(),
            )),
        }
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct PrerequisiteList(pub(super) Vec<String>);

impl FromMeta for PrerequisiteList {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        let syn::Expr::Array(array) = expr else {
            return Err(darling::Error::unexpected_expr_type(expr));
        };
        array
            .elems
            .iter()
            .map(Self::parse_prereq_expr)
            .collect::<darling::Result<Vec<_>>>()
            .map(Self)
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(Self::parse_nested_prereq)
            .collect::<darling::Result<Vec<_>>>()
            .map(Self)
    }
}

impl PrerequisiteList {
    fn parse_prereq_expr(expr: &syn::Expr) -> darling::Result<String> {
        match expr {
            syn::Expr::Path(path) => Ok(compact_type_tokens(quote!(#path))),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => Ok(lit.value()),
            syn::Expr::Call(call) => Self::parse_parent_call(call),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
    }

    fn parse_parent_call(call: &syn::ExprCall) -> darling::Result<String> {
        let syn::Expr::Path(callee) = &*call.func else {
            return Err(darling::Error::unexpected_expr_type(&call.func));
        };
        if !callee.path.is_ident("parent") || call.args.len() != 1 {
            return Err(darling::Error::custom(
                "expected 'parent(<dep>)' with a single dependency identifier",
            ));
        }
        let Some(syn::Expr::Path(arg_path)) = call.args.first() else {
            return Err(darling::Error::custom(
                "parent(...) argument must be a dependency identifier",
            ));
        };
        Ok(format!(
            "parent({})",
            compact_type_tokens(quote!(#arg_path))
        ))
    }

    fn parse_nested_prereq(item: &NestedMeta) -> darling::Result<String> {
        match item {
            NestedMeta::Meta(syn::Meta::Path(path)) => Ok(compact_type_tokens(quote!(#path))),
            NestedMeta::Lit(syn::Lit::Str(lit)) => Ok(lit.value()),
            _ => Err(darling::Error::unexpected_type("prerequisite identifier")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct DependencyAttrMeta {
    #[darling(default)]
    pub deps: PrerequisiteList,
    #[darling(default)]
    pub is_async: bool,
    #[darling(default)]
    pub constructor: Option<FunctionName>,
    #[darling(default)]
    pub builder: Option<FunctionName>,
}

/// The value of `constructor = ...` or `builder = ...` as the user wrote it.
/// Only a bare function name (`constructor = connect`) is accepted; the
/// field parser turns the other shapes into errors that name the field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FunctionName {
    Bare(String),
    Quoted(String),
    Invalid(String),
}

impl FromMeta for FunctionName {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        Ok(match expr {
            syn::Expr::Path(path) if path.qself.is_none() => path.path.get_ident().map_or_else(
                || Self::Invalid(quote!(#path).to_string()),
                |ident| Self::Bare(ident.to_string()),
            ),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Self::Quoted(text.value()),
            other => Self::Invalid(quote!(#other).to_string()),
        })
    }
}

impl DependencyAttrMeta {
    pub(super) fn from_attribute(attr: &syn::Attribute) -> Result<Self, DomainError> {
        match &attr.meta {
            syn::Meta::Path(_) => Ok(Self::default()),
            syn::Meta::List(list) => Self::from_tokens(list.tokens.clone()),
            syn::Meta::NameValue(_) => Err(DomainError::InvalidConfiguration(
                "lifecycle attribute does not accept name-value assignment".to_string(),
            )),
        }
    }

    fn from_tokens(tokens: TokenStream) -> Result<Self, DomainError> {
        let normalized = normalize_async_keyword(tokens);
        let nested = NestedMeta::parse_meta_list(normalized).map_err(invalid_config)?;
        Self::from_list(&nested).map_err(invalid_config)
    }
}

fn normalize_async_keyword(tokens: TokenStream) -> TokenStream {
    tokens
        .into_iter()
        .map(|tree| match tree {
            TokenTree::Ident(ident) if ident == "async" => {
                TokenTree::Ident(Ident::new("is_async", ident.span()))
            }
            tree => tree,
        })
        .collect()
}

fn invalid_config(err: impl std::fmt::Display) -> DomainError {
    DomainError::InvalidConfiguration(err.to_string())
}

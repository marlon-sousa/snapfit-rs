//! Adapter: the attribute arguments as darling reads them, before they are checked.

use darling::FromMeta;
use darling::ast::NestedMeta;
use derive_more::Display;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;
use syn::punctuated::Punctuated;

use super::type_inspector::compact_type_tokens;
use crate::domain::AttributeProblem;

mod prerequisites;
#[cfg(test)]
mod tests;

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
    #[darling(default, rename = "async")]
    pub(super) is_async: bool,
    #[darling(default)]
    pub(super) build: Option<BuildMeta>,
}

/// The value of `build = ...` or `leaf_build = ...` as the user wrote it: one
/// of the words darling knows, another word, `constructor(...)`, or
/// `builder(...)`, which is refused as planned whatever its arguments. Every
/// other shape is turned into an error when it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BuildMeta {
    Word(BuildWord),
    UnknownWord(syn::Ident),
    Builder,
    Constructor(FunctionNameMeta),
    Quoted(String),
    Invalid(String),
}

/// The bare words `build` and `leaf_build` take, matched by darling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromMeta, Display)]
#[darling(rename_all = "snake_case")]
#[display(rename_all = "snake_case")]
pub(super) enum BuildWord {
    Default,
    Factory,
    Builder,
    /// `constructor` without the function's name in brackets.
    Constructor,
}

impl FromMeta for BuildMeta {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        Ok(match expr {
            syn::Expr::Path(path) if path.qself.is_none() => path.path.get_ident().map_or_else(
                || Self::Invalid(quote!(#path).to_string()),
                |ident| {
                    BuildWord::from_string(&ident.to_string())
                        .map_or_else(|_| Self::UnknownWord(ident.clone()), Self::Word)
                },
            ),
            syn::Expr::Call(call) => match called_word(&call.func) {
                Some(BuildWord::Constructor) => {
                    Self::Constructor(FunctionNameMeta::from_arguments(&call.args))
                }
                Some(BuildWord::Builder) => Self::Builder,
                _ => Self::Invalid(quote!(#call).to_string()),
            },
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Self::Quoted(text.value()),
            other => Self::Invalid(quote!(#other).to_string()),
        })
    }
}

/// The word a call such as `constructor(connect)` is made of, when it is one.
fn called_word(func: &syn::Expr) -> Option<BuildWord> {
    match func {
        syn::Expr::Path(path) => path
            .path
            .get_ident()
            .and_then(|ident| BuildWord::from_string(&ident.to_string()).ok()),
        _ => None,
    }
}

/// The argument of `constructor(...)`, with its span. Only one bare function
/// name (`constructor(connect)`) is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FunctionNameMeta {
    Bare(syn::Ident),
    Quoted(String),
    Invalid(String),
}

impl FunctionNameMeta {
    fn from_arguments(args: &Punctuated<syn::Expr, Token![,]>) -> Self {
        match (args.first(), args.len()) {
            (Some(argument), 1) => Self::from_argument(argument),
            _ => Self::Invalid(quote!(#args).to_string()),
        }
    }

    fn from_argument(expr: &syn::Expr) -> Self {
        match expr {
            syn::Expr::Path(path) if path.qself.is_none() => path.path.get_ident().map_or_else(
                || Self::Invalid(quote!(#path).to_string()),
                |ident| Self::Bare(ident.clone()),
            ),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Self::Quoted(text.value()),
            other => Self::Invalid(quote!(#other).to_string()),
        }
    }
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
        let nested = NestedMeta::parse_meta_list(tokens).map_err(unreadable)?;
        Self::from_list(&nested).map_err(unreadable)
    }
}

fn unreadable(err: impl std::fmt::Display) -> AttributeProblem {
    AttributeProblem::Unreadable(err.to_string())
}

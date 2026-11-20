//! Adapter: the attribute arguments as darling reads them, before they are checked.

use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;
use quote::quote;

use super::type_inspector::compact_type_tokens;
use crate::domain::AttributeProblem;

mod build_meta;
mod expressions;
mod fallible_meta;
mod implementation_meta;
mod prerequisites;
#[cfg(test)]
mod tests;

pub(super) use build_meta::{BuildMeta, BuildWord, FunctionNameMeta};
pub(super) use expressions::single_word;
use expressions::ungrouped;
pub(super) use fallible_meta::FallibleMeta;
use implementation_meta::FieldSettings;
pub(super) use implementation_meta::ImplementationMeta;
pub(super) use prerequisites::{PrerequisiteEntry, PrerequisiteList, PrerequisiteMeta};

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

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct DependencyAttrMeta {
    #[darling(default)]
    pub(super) deps: PrerequisiteList,
    #[darling(default, rename = "async")]
    pub(super) is_async: bool,
    #[darling(default)]
    pub(super) build: Option<BuildMeta>,
    #[darling(default)]
    pub(super) fallible: Option<FallibleMeta>,
    /// Filled by `FieldSettings`, never by darling.
    #[darling(skip)]
    pub(super) implementations: Vec<ImplementationMeta>,
    /// Never filled: `FieldSettings` takes every `implementation` out
    /// first. Without it, darling's help for a misspelt setting would leave
    /// `implementation` out of the settings it offers.
    #[darling(default, rename = "implementation")]
    implementation_name: Option<darling::util::Ignored>,
}

impl DependencyAttrMeta {
    pub(super) fn from_attribute(attr: &syn::Attribute) -> Result<Self, UnreadableAttribute> {
        match &attr.meta {
            syn::Meta::Path(_) => Ok(Self::default()),
            syn::Meta::List(list) => Self::from_tokens(list.tokens.clone()),
            syn::Meta::NameValue(_) => Err(UnreadableAttribute::alone(AttributeProblem::Assigned)),
        }
    }

    fn from_tokens(tokens: TokenStream) -> Result<Self, UnreadableAttribute> {
        let FieldSettings {
            settings,
            implementations,
        } = syn::parse2(tokens)
            .map_err(unreadable)
            .map_err(UnreadableAttribute::alone)?;
        let meta = match Self::from_list(&settings) {
            Ok(meta) => meta,
            Err(err) => {
                return Err(UnreadableAttribute {
                    problem: unreadable(err),
                    bad_deps: bad_deps(&settings),
                    implementations,
                });
            }
        };
        Ok(Self {
            implementations,
            ..meta
        })
    }
}

/// A field attribute that does not read, with every bad `deps` entry and
/// every `implementation` it holds, which darling's problem with the rest
/// of it leaves out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UnreadableAttribute {
    pub(super) problem: AttributeProblem,
    pub(super) bad_deps: Vec<AttributeProblem>,
    pub(super) implementations: Vec<ImplementationMeta>,
}

impl UnreadableAttribute {
    fn alone(problem: AttributeProblem) -> Self {
        Self {
            problem,
            bad_deps: Vec::new(),
            implementations: Vec::new(),
        }
    }
}

fn bad_deps(nested: &[NestedMeta]) -> Vec<AttributeProblem> {
    nested
        .iter()
        .filter_map(deps_setting)
        .flat_map(|deps| deps.0)
        .filter_map(Result::err)
        .collect()
}

/// The entries of a `deps` setting; one that does not read at all is
/// already in darling's problem.
fn deps_setting(item: &NestedMeta) -> Option<PrerequisiteList> {
    match item {
        NestedMeta::Meta(meta) if meta.path().is_ident("deps") => {
            PrerequisiteList::from_meta(meta).ok()
        }
        _ => None,
    }
}

fn unreadable(err: impl std::fmt::Display) -> AttributeProblem {
    AttributeProblem::Unreadable(err.to_string())
}

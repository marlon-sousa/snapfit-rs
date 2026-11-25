//! Adapter: the attribute arguments as darling reads them, before they are checked.

use darling::FromMeta;
use darling::ast::NestedMeta;
use derive_new::new;
use proc_macro2::TokenStream;
use quote::quote;

use super::type_inspector::compact_type_tokens;
use crate::domain::ArgumentProblem;

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
pub(super) use implementation_meta::{ImplementationEntry, ImplementationMeta};
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
    pub(super) fn from_tokens(tokens: TokenStream) -> Result<Self, UnreadableAttribute<Self>> {
        read_list(tokens)
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
    pub(super) fn from_attribute(attr: &syn::Attribute) -> Result<Self, UnreadableAttribute<Self>> {
        match &attr.meta {
            syn::Meta::Path(_) => Ok(Self::default()),
            syn::Meta::List(list) => Self::from_tokens(list.tokens.clone()),
            syn::Meta::NameValue(_) => Err(UnreadableAttribute::new(ArgumentProblem::Assigned)),
        }
    }

    /// When darling refuses the rest, every `implementation` follows the
    /// rescued settings as one value, so that a repeated one is still refused.
    fn from_tokens(tokens: TokenStream) -> Result<Self, UnreadableAttribute<Self>> {
        let FieldSettings {
            settings,
            implementations,
        } = syn::parse2(tokens)
            .map_err(unreadable)
            .map_err(UnreadableAttribute::new)?;
        match read_nested::<Self>(&settings) {
            Ok(meta) => Ok(Self {
                implementations,
                ..meta
            }),
            Err(mut unreadable) => {
                if !implementations.is_empty() {
                    unreadable.settings.push(Self {
                        implementations,
                        ..Self::default()
                    });
                }
                Err(unreadable)
            }
        }
    }
}

/// An attribute that does not read, with each of its settings that reads on
/// its own, whose problems darling's problem with the rest leaves out.
#[derive(new, Debug)]
pub(super) struct UnreadableAttribute<Settings> {
    pub(super) problem: ArgumentProblem,
    /// One value per readable setting, holding that setting alone, in the order written.
    #[new(default)]
    pub(super) settings: Vec<Settings>,
}

fn read_list<Settings: FromMeta>(
    tokens: TokenStream,
) -> Result<Settings, UnreadableAttribute<Settings>> {
    let nested = NestedMeta::parse_meta_list(tokens)
        .map_err(unreadable)
        .map_err(UnreadableAttribute::new)?;
    read_nested(&nested)
}

fn read_nested<Settings: FromMeta>(
    nested: &[NestedMeta],
) -> Result<Settings, UnreadableAttribute<Settings>> {
    Settings::from_list(nested).map_err(|err| UnreadableAttribute {
        problem: unreadable(err),
        settings: nested.iter().filter_map(read_alone).collect(),
    })
}

/// The setting read as if it were the only one; a setting that does not
/// read even alone is already in darling's problem. This holds only while
/// every field of the settings is `#[darling(default)]`.
fn read_alone<Settings: FromMeta>(setting: &NestedMeta) -> Option<Settings> {
    Settings::from_list(std::slice::from_ref(setting)).ok()
}

fn unreadable(err: impl std::fmt::Display) -> ArgumentProblem {
    ArgumentProblem::Unreadable(err.to_string())
}

//! Adapter: the entries of `deps = [...]` as darling reads them.

use darling::FromMeta;
use darling::ast::NestedMeta;
use quote::{ToTokens, quote};

use super::super::type_inspector::compact_type_tokens;
use super::expressions::{single_word, ungrouped};
use crate::domain::AttributeProblem;

/// Each entry of `deps = [...]` in the order written, or what is wrong with it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) struct PrerequisiteList(
    pub(in crate::adapters::inbound) Vec<PrerequisiteEntry>,
);

/// One entry of `deps = [...]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum PrerequisiteMeta {
    /// Another field of the same container, with the word it is written as,
    /// with its span, when it is one.
    Sibling {
        name: String,
        word: Option<syn::Ident>,
    },
    /// `label = name`: another field of the same container, passed to the
    /// builder setter `label`.
    Labelled { label: syn::Ident, name: String },
    /// `parent(name)`, a dependency of the parent container.
    Parent(String),
}

pub(in crate::adapters::inbound) type PrerequisiteEntry =
    Result<PrerequisiteMeta, AttributeProblem>;

impl FromMeta for PrerequisiteList {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::refusing("deps"))
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        Ok(Self::from_value(expr))
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        Ok(Self(items.iter().map(entry_of_nested).collect()))
    }
}

impl PrerequisiteList {
    fn from_value(expr: &syn::Expr) -> Self {
        match ungrouped(expr) {
            syn::Expr::Array(array) => Self(array.elems.iter().map(entry_of_expr).collect()),
            other => Self::refusing(&format!("deps = {}", written(other))),
        }
    }

    fn refusing(written: &str) -> Self {
        Self(vec![Err(AttributeProblem::DepsNotAList(
            written.to_string(),
        ))])
    }
}

impl PrerequisiteMeta {
    pub(in crate::adapters::inbound) fn into_name(self) -> String {
        match self {
            Self::Sibling { name, .. } | Self::Labelled { name, .. } | Self::Parent(name) => name,
        }
    }

    fn sibling(path: &syn::Path) -> Self {
        Self::Sibling {
            name: compact_type_tokens(quote!(#path)),
            word: path.get_ident().cloned(),
        }
    }

    fn quoted_sibling(lit: &syn::LitStr) -> Self {
        Self::Sibling {
            name: lit.value(),
            word: lit.parse().ok(),
        }
    }

    /// `label = name`, both single words, or the problem with the whole entry.
    fn labelled(
        label: Option<&syn::Ident>,
        name: Option<&syn::Ident>,
        entry: impl ToTokens,
    ) -> PrerequisiteEntry {
        match (label, name) {
            (Some(label), Some(name)) => Ok(Self::Labelled {
                label: label.clone(),
                name: name.to_string(),
            }),
            _ => Err(AttributeProblem::NotALabel(written(entry))),
        }
    }
}

fn entry_of_expr(expr: &syn::Expr) -> PrerequisiteEntry {
    match ungrouped(expr) {
        syn::Expr::Path(path) => Ok(PrerequisiteMeta::sibling(&path.path)),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => Ok(PrerequisiteMeta::quoted_sibling(lit)),
        syn::Expr::Assign(assign) => PrerequisiteMeta::labelled(
            single_word(&assign.left),
            single_word(&assign.right),
            assign,
        ),
        syn::Expr::Call(call) if single_word(&call.func).is_some_and(|word| word == "parent") => {
            parent_entry(call)
        }
        other => Err(AttributeProblem::NotAField(written(other))),
    }
}

/// `parent(name)` with exactly one name, which may be a path.
fn parent_entry(call: &syn::ExprCall) -> PrerequisiteEntry {
    match (call.args.first().map(ungrouped), call.args.len()) {
        (Some(syn::Expr::Path(argument)), 1) => Ok(PrerequisiteMeta::Parent(compact_type_tokens(
            quote!(#argument),
        ))),
        _ => Err(AttributeProblem::NotAParentDependency(written(call))),
    }
}

fn entry_of_nested(item: &NestedMeta) -> PrerequisiteEntry {
    match item {
        NestedMeta::Meta(syn::Meta::Path(path)) => Ok(PrerequisiteMeta::sibling(path)),
        NestedMeta::Meta(syn::Meta::NameValue(pair)) => {
            PrerequisiteMeta::labelled(pair.path.get_ident(), single_word(&pair.value), pair)
        }
        NestedMeta::Lit(syn::Lit::Str(lit)) => Ok(PrerequisiteMeta::quoted_sibling(lit)),
        other => Err(AttributeProblem::NotAField(written(other))),
    }
}

/// The tokens as the user wrote them, for a message.
fn written(tokens: impl ToTokens) -> String {
    compact_type_tokens(tokens.to_token_stream())
}

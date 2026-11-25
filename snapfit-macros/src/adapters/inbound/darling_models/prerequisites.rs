//! Adapter: the entries of `deps = [...]` as darling reads them.

use darling::FromMeta;
use quote::{ToTokens, quote};
use syn::ext::IdentExt;

use super::super::type_inspector::compact_type_tokens;
use super::expressions::{single_word, ungrouped};
use crate::domain::ArgumentProblem;

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

pub(in crate::adapters::inbound) type PrerequisiteEntry = Result<PrerequisiteMeta, ArgumentProblem>;

impl FromMeta for PrerequisiteList {
    /// `deps(...)` is refused whatever its parentheses hold, before darling
    /// would read them as settings and stop at one it cannot read.
    fn from_meta(meta: &syn::Meta) -> darling::Result<Self> {
        Ok(match meta {
            syn::Meta::Path(_) => Self::refusing("deps"),
            syn::Meta::List(list) => Self::in_parentheses(&list.tokens),
            syn::Meta::NameValue(pair) => Self::from_value(&pair.value),
        })
    }
}

impl PrerequisiteList {
    fn from_value(expr: &syn::Expr) -> Self {
        match ungrouped(expr) {
            syn::Expr::Array(array) => Self(array.elems.iter().map(entry_of_expr).collect()),
            other => Self::refusing(&format!("deps = {}", written(other))),
        }
    }

    fn in_parentheses(tokens: &proc_macro2::TokenStream) -> Self {
        Self(vec![Err(ArgumentProblem::DepsInParentheses(written(
            tokens,
        )))])
    }

    fn refusing(written: &str) -> Self {
        Self(vec![Err(ArgumentProblem::DepsNotAList(
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
            name: path.get_ident().map_or_else(|| written(path), name_of_word),
            word: path.get_ident().cloned(),
        }
    }

    fn quoted_sibling(lit: &syn::LitStr) -> Self {
        let word: Option<syn::Ident> = lit.parse().ok();
        Self::Sibling {
            name: word.as_ref().map_or_else(|| lit.value(), name_of_word),
            word,
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
                name: name_of_word(name),
            }),
            _ => Err(ArgumentProblem::NotALabel(written(entry))),
        }
    }
}

/// The name a word stands for, without the `r#` of a raw identifier, which
/// Rust reads as the same identifier.
fn name_of_word(word: &syn::Ident) -> String {
    word.unraw().to_string()
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
        other => Err(ArgumentProblem::NotAField(written(other))),
    }
}

/// `parent(name)` with exactly one name, which may be a path.
fn parent_entry(call: &syn::ExprCall) -> PrerequisiteEntry {
    match (call.args.first().map(ungrouped), call.args.len()) {
        (Some(syn::Expr::Path(argument)), 1) => Ok(PrerequisiteMeta::Parent(compact_type_tokens(
            quote!(#argument),
        ))),
        _ => Err(ArgumentProblem::NotAParentDependency(written(call))),
    }
}

/// The tokens as the user wrote them, for a message.
fn written(tokens: impl ToTokens) -> String {
    compact_type_tokens(tokens.to_token_stream())
}

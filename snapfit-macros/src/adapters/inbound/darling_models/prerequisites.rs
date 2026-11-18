//! Adapter: the entries of `deps = [...]` as darling reads them.

use darling::FromMeta;
use darling::ast::NestedMeta;
use quote::quote;
use strum::EnumIs;

use super::super::type_inspector::compact_type_tokens;
use super::single_word;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) struct PrerequisiteList(
    pub(in crate::adapters::inbound) Vec<PrerequisiteMeta>,
);

/// One entry of `deps = [...]`.
#[derive(EnumIs, Debug, Clone, PartialEq, Eq)]
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

    /// `label = name`, both single words.
    fn labelled(label: Option<&syn::Ident>, name: Option<&syn::Ident>) -> darling::Result<Self> {
        match (label, name) {
            (Some(label), Some(name)) => Ok(Self::Labelled {
                label: label.clone(),
                name: name.to_string(),
            }),
            _ => Err(darling::Error::custom(
                "expected 'setter = dependency', each a single identifier",
            )),
        }
    }
}

impl PrerequisiteList {
    fn parse_prereq_expr(expr: &syn::Expr) -> darling::Result<PrerequisiteMeta> {
        match expr {
            syn::Expr::Path(path) => Ok(PrerequisiteMeta::sibling(&path.path)),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => Ok(PrerequisiteMeta::quoted_sibling(lit)),
            syn::Expr::Assign(assign) => {
                PrerequisiteMeta::labelled(single_word(&assign.left), single_word(&assign.right))
            }
            syn::Expr::Call(call) => Self::parse_parent_call(call),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
    }

    fn parse_parent_call(call: &syn::ExprCall) -> darling::Result<PrerequisiteMeta> {
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
        Ok(PrerequisiteMeta::Parent(compact_type_tokens(quote!(
            #arg_path
        ))))
    }

    fn parse_nested_prereq(item: &NestedMeta) -> darling::Result<PrerequisiteMeta> {
        match item {
            NestedMeta::Meta(syn::Meta::Path(path)) => Ok(PrerequisiteMeta::sibling(path)),
            NestedMeta::Meta(syn::Meta::NameValue(pair)) => {
                PrerequisiteMeta::labelled(pair.path.get_ident(), single_word(&pair.value))
            }
            NestedMeta::Lit(syn::Lit::Str(lit)) => Ok(PrerequisiteMeta::quoted_sibling(lit)),
            _ => Err(darling::Error::unexpected_type("prerequisite identifier")),
        }
    }
}

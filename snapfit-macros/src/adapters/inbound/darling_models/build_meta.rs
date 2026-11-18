//! Adapter: the value of `build = ...` and `leaf_build = ...` as darling reads it.

use darling::FromMeta;
use quote::quote;
use syn::Token;
use syn::punctuated::Punctuated;

/// The value of `build = ...` or `leaf_build = ...` as the user wrote it: a
/// bare word, `constructor(...)`, or `builder(...)` with its settings. Every
/// other shape is turned into an error when it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum BuildMeta {
    /// One of the words darling knows, with the word as written, for its span.
    Word {
        word: BuildWord,
        written: syn::Ident,
    },
    UnknownWord(syn::Ident),
    /// `builder(...)`: the word, for the span of its default functions, and
    /// its settings as written.
    Builder {
        word: syn::Ident,
        settings: Vec<syn::Expr>,
    },
    Constructor(FunctionNameMeta),
    Quoted(String),
    Invalid(String),
}

/// The bare words `build` and `leaf_build` take, matched by darling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromMeta)]
#[darling(rename_all = "snake_case")]
pub(in crate::adapters::inbound) enum BuildWord {
    Default,
    Factory,
    Builder,
    /// `constructor` without the function's name in brackets.
    Constructor,
}

impl FromMeta for BuildMeta {
    /// darling's derive reads a unit variant only from a quoted string and
    /// does not allow `from_expr` beside one, so the bare word is handed to
    /// the derived [`BuildWord`] here.
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        Ok(match expr {
            syn::Expr::Path(_) => single_word(expr)
                .map_or_else(|| Self::Invalid(quote!(#expr).to_string()), Self::from_word),
            syn::Expr::Call(call) => Self::from_call(call),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Self::Quoted(text.value()),
            other => Self::Invalid(quote!(#other).to_string()),
        })
    }
}

impl BuildMeta {
    fn from_word(written: &syn::Ident) -> Self {
        BuildWord::from_string(&written.to_string()).map_or_else(
            |_| Self::UnknownWord(written.clone()),
            |word| Self::Word {
                word,
                written: written.clone(),
            },
        )
    }

    /// `constructor(...)` or `builder(...)`; any other call is not a build.
    fn from_call(call: &syn::ExprCall) -> Self {
        match single_word(&call.func) {
            Some(word) if word == "constructor" => {
                Self::Constructor(FunctionNameMeta::from_arguments(&call.args))
            }
            Some(word) if word == "builder" => Self::Builder {
                word: word.clone(),
                settings: call.args.iter().cloned().collect(),
            },
            _ => Self::Invalid(quote!(#call).to_string()),
        }
    }
}

/// The expression as a single word, such as the function a call names or
/// either side of `start = make`.
pub(in crate::adapters::inbound) fn single_word(expr: &syn::Expr) -> Option<&syn::Ident> {
    match expr {
        syn::Expr::Path(path) if path.qself.is_none() => path.path.get_ident(),
        _ => None,
    }
}

/// The argument of `constructor(...)`, with its span. Only one bare function
/// name (`constructor(connect)`) is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum FunctionNameMeta {
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
            syn::Expr::Path(_) => single_word(expr).map_or_else(
                || Self::Invalid(quote!(#expr).to_string()),
                |word| Self::Bare(word.clone()),
            ),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Self::Quoted(text.value()),
            other => Self::Invalid(quote!(#other).to_string()),
        }
    }
}

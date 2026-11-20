//! Adapter: each `implementation` setting of a field attribute, read apart
//! from the settings darling reads.

use darling::ast::NestedMeta;
use proc_macro2::{TokenStream, TokenTree};
use strum::{EnumIs, EnumTryAs};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use super::super::type_inspector::compact_type_tokens;

/// One `implementation` setting, or the shape it was written in instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum ImplementationMeta {
    Type(syn::Type),
    /// `implementation` with no type: bare, or with an empty value.
    Bare,
    /// `implementation(...)`, with what its parentheses hold.
    InParentheses(String),
    /// `implementation = "..."`, with the string's own text.
    Quoted(String),
    /// A value that is no type the container can build, as written.
    NotAType(String),
}

/// The settings of a field attribute in the order written, with each
/// `implementation` taken out of them, whose value, a type such as
/// `Cache<Order>`, darling cannot read.
pub(super) struct FieldSettings {
    pub(super) settings: Vec<NestedMeta>,
    pub(super) implementations: Vec<ImplementationMeta>,
}

#[derive(EnumIs, EnumTryAs)]
enum Setting {
    Implementation(ImplementationMeta),
    Other(NestedMeta),
}

impl Parse for FieldSettings {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (implementations, others): (Vec<_>, Vec<_>) =
            Punctuated::<Setting, syn::Token![,]>::parse_terminated(input)?
                .into_iter()
                .partition(Setting::is_implementation);
        Ok(Self {
            settings: others
                .into_iter()
                .filter_map(Setting::try_as_other)
                .collect(),
            implementations: implementations
                .into_iter()
                .filter_map(Setting::try_as_implementation)
                .collect(),
        })
    }
}

impl Parse for Setting {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if !starts_implementation(input) {
            return input.parse().map(Self::Other);
        }
        input.parse::<syn::Ident>()?;
        let meta = if input.peek(syn::token::Paren) {
            let inner;
            syn::parenthesized!(inner in input);
            in_parentheses(inner.parse()?)
        } else if input.parse::<Option<syn::Token![=]>>()?.is_some() {
            implementation_value(input)?
        } else {
            ImplementationMeta::Bare
        };
        Ok(Self::Implementation(meta))
    }
}

fn starts_implementation(input: ParseStream<'_>) -> bool {
    input
        .fork()
        .parse::<syn::Ident>()
        .is_ok_and(|word| word == "implementation")
}

fn in_parentheses(tokens: TokenStream) -> ImplementationMeta {
    if tokens.is_empty() {
        ImplementationMeta::Bare
    } else {
        ImplementationMeta::InParentheses(compact_type_tokens(tokens))
    }
}

/// The value after `implementation =`, up to the next setting.
fn implementation_value(input: ParseStream<'_>) -> syn::Result<ImplementationMeta> {
    if let Some(quoted) = input.parse::<Option<syn::LitStr>>()? {
        return Ok(match quoted.value() {
            text if text.trim().is_empty() => ImplementationMeta::Bare,
            text => ImplementationMeta::Quoted(text),
        });
    }
    if reads_as_a_whole_type(input) {
        return input.parse().map(buildable);
    }
    rest_of_setting(input).map(not_a_type)
}

/// Whether the value is one type that ends the setting, so that a comma
/// inside it, as in `HashMap<_, String>`, is not read as the setting's end.
fn reads_as_a_whole_type(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    fork.parse::<syn::Type>().is_ok() && ends_setting(&fork)
}

fn buildable(ty: syn::Type) -> ImplementationMeta {
    if is_buildable(&ty) {
        ImplementationMeta::Type(ty)
    } else {
        not_a_type(quote::quote!(#ty))
    }
}

fn not_a_type(written: TokenStream) -> ImplementationMeta {
    if written.is_empty() {
        ImplementationMeta::Bare
    } else {
        ImplementationMeta::NotAType(compact_type_tokens(written))
    }
}

fn ends_setting(input: ParseStream<'_>) -> bool {
    input.is_empty() || input.peek(syn::Token![,])
}

/// A type the container can call a constructor of: not a trait object,
/// and with no `impl Trait` or `_` anywhere in it, which generated code
/// cannot name.
fn is_buildable(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Group(group) => is_buildable(&group.elem),
        syn::Type::Paren(paren) => is_buildable(&paren.elem),
        syn::Type::TraitObject(_) => false,
        _ => names_every_type(quote::quote!(#ty)),
    }
}

fn names_every_type(tokens: TokenStream) -> bool {
    tokens.into_iter().all(names_a_type)
}

fn names_a_type(tree: TokenTree) -> bool {
    match tree {
        TokenTree::Ident(word) => word != "impl" && word != "_",
        TokenTree::Group(group) => names_every_type(group.stream()),
        TokenTree::Punct(_) | TokenTree::Literal(_) => true,
    }
}

/// The tokens up to the next setting, as written.
fn rest_of_setting(input: ParseStream<'_>) -> syn::Result<TokenStream> {
    let mut written = TokenStream::new();
    while !ends_setting(input) {
        written.extend([input.parse::<TokenTree>()?]);
    }
    Ok(written)
}

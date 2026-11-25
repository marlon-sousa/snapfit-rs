//! Adapter: each `implementation` setting of a field attribute, read apart
//! from the settings darling reads.

use darling::ast::NestedMeta;
use proc_macro2::{TokenStream, TokenTree};
use strum::{EnumIs, EnumTryAs};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use super::super::type_inspector::{compact_type_tokens, is_buildable};

/// One `implementation` setting: a type, or a list of types in square brackets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum ImplementationMeta {
    One(ImplementationEntry),
    List(Vec<ImplementationEntry>),
}

/// One type an `implementation` setting names, or the shape it was written in instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum ImplementationEntry {
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
            ImplementationMeta::One(in_parentheses(inner.parse()?))
        } else if input.parse::<Option<syn::Token![=]>>()?.is_some() {
            implementation_value(input)?
        } else {
            ImplementationMeta::One(ImplementationEntry::Bare)
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

fn in_parentheses(tokens: TokenStream) -> ImplementationEntry {
    if tokens.is_empty() {
        ImplementationEntry::Bare
    } else {
        ImplementationEntry::InParentheses(compact_type_tokens(tokens))
    }
}

/// The value after `implementation =`, up to the next setting: a list when
/// it is one pair of square brackets, and one type otherwise.
fn implementation_value(input: ParseStream<'_>) -> syn::Result<ImplementationMeta> {
    if !is_a_list(input) {
        return entry_value(input).map(ImplementationMeta::One);
    }
    let inner;
    syn::bracketed!(inner in input);
    let entries = inner.parse_terminated(entry_value, syn::Token![,])?;
    Ok(ImplementationMeta::List(entries.into_iter().collect()))
}

fn is_a_list(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    input.peek(syn::token::Bracket) && fork.parse::<TokenTree>().is_ok() && ends_setting(&fork)
}

/// One type, up to the next comma.
fn entry_value(input: ParseStream<'_>) -> syn::Result<ImplementationEntry> {
    if let Some(quoted) = input.parse::<Option<syn::LitStr>>()? {
        return Ok(match quoted.value() {
            text if text.trim().is_empty() => ImplementationEntry::Bare,
            text => ImplementationEntry::Quoted(text),
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

fn buildable(ty: syn::Type) -> ImplementationEntry {
    if is_buildable(&ty) {
        ImplementationEntry::Type(ty)
    } else {
        not_a_type(quote::quote!(#ty))
    }
}

fn not_a_type(written: TokenStream) -> ImplementationEntry {
    if written.is_empty() {
        ImplementationEntry::Bare
    } else {
        ImplementationEntry::NotAType(compact_type_tokens(written))
    }
}

fn ends_setting(input: ParseStream<'_>) -> bool {
    input.is_empty() || input.peek(syn::Token![,])
}

/// The tokens up to the next setting, as written.
fn rest_of_setting(input: ParseStream<'_>) -> syn::Result<TokenStream> {
    let mut written = TokenStream::new();
    while !ends_setting(input) {
        written.extend([input.parse::<TokenTree>()?]);
    }
    Ok(written)
}

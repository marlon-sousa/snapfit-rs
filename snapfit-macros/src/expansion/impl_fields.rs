//! PROTOTYPE, not for merging: rewrites `impl Trait` in field types before
//! the container is read.
//!
//! - A field typed `impl Trait` with `implementation = X` becomes `X`, and
//!   its `implementation` setting is taken out.
//! - In every other field, an `impl Trait` becomes the implementation of
//!   the field, among its deps (followed further down), whose type was
//!   `impl Trait` for the same trait.

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit_mut::VisitMut;

struct Declared {
    trait_key: String,
    implementation: syn::Type,
}

pub(super) fn rewrite(item: &mut syn::ItemStruct) -> TokenStream {
    let mut mentions = TokenStream::new();
    let syn::Fields::Named(fields) = &mut item.fields else { return mentions };
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    let mut declared: HashMap<String, Declared> = HashMap::new();
    for field in fields.named.iter_mut() {
        let Some(name) = field.ident.as_ref().map(ToString::to_string) else { continue };
        let Some(attr) = field.attrs.iter_mut().find(|a| a.path().is_ident("singleton")) else {
            continue;
        };
        let syn::Meta::List(list) = &mut attr.meta else { continue };
        let settings = split_settings(list.tokens.clone());
        deps.insert(name.clone(), settings.iter().find_map(|s| dep_names(s)).unwrap_or_default());
        let syn::Type::ImplTrait(impl_trait) = &field.ty else { continue };
        let Some(implementation) = settings.iter().find_map(|s| implementation_of(s)) else {
            continue;
        };
        let trait_key = trait_key(impl_trait);
        let span = field.ty.to_token_stream().into_iter().next().map_or(Span::call_site(), |t| t.span());
        field.ty = respanned(&implementation, span);
        mentions.extend(quote::quote!(const _: ::core::marker::PhantomData<#implementation> = ::core::marker::PhantomData;));
        list.tokens = join_settings(settings.into_iter().filter(|s| implementation_of(s).is_none()));
        declared.insert(name, Declared { trait_key, implementation });
    }
    for field in fields.named.iter_mut() {
        let Some(name) = field.ident.as_ref().map(ToString::to_string) else { continue };
        let reachable = reachable(&name, &deps);
        let mut replacer = Replacer { reachable: reachable.iter().filter_map(|d| declared.get(d)).collect() };
        replacer.visit_type_mut(&mut field.ty);
    }
    mentions
}

struct Replacer<'a> {
    reachable: Vec<&'a Declared>,
}

impl VisitMut for Replacer<'_> {
    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        if let syn::Type::ImplTrait(impl_trait) = ty {
            let key = trait_key(impl_trait);
            let found: Vec<_> = self.reachable.iter().filter(|d| d.trait_key == key).collect();
            if let [only] = found.as_slice() {
                let span = ty.to_token_stream().into_iter().next().map_or(Span::call_site(), |t| t.span());
                *ty = respanned(&only.implementation, span);
            }
            return;
        }
        syn::visit_mut::visit_type_mut(self, ty);
    }
}

/// The deps of `name`, then theirs, and so on.
fn reachable(name: &str, deps: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut todo: Vec<String> = deps.get(name).cloned().unwrap_or_default();
    while let Some(next) = todo.pop() {
        if seen.contains(&next) || next == name { continue }
        todo.extend(deps.get(&next).cloned().unwrap_or_default());
        seen.push(next);
    }
    seen
}

fn trait_key(impl_trait: &syn::TypeImplTrait) -> String {
    impl_trait
        .bounds
        .iter()
        .find_map(|bound| match bound {
            syn::TypeParamBound::Trait(t)
                if !["Send", "Sync", "Unpin"].iter().any(|auto| t.path.is_ident(auto)) =>
            {
                Some(t.path.to_token_stream().to_string())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// The implementation's tokens, spanned at the field's `impl` word, so that
/// an editor maps the field's text to the type that replaced it.
fn respanned(ty: &syn::Type, span: Span) -> syn::Type {
    fn walk(stream: TokenStream, span: Span) -> TokenStream {
        stream
            .into_iter()
            .map(|tree| match tree {
                TokenTree::Group(group) => {
                    let mut g = proc_macro2::Group::new(group.delimiter(), walk(group.stream(), span));
                    g.set_span(span);
                    TokenTree::Group(g)
                }
                mut other => {
                    other.set_span(span);
                    other
                }
            })
            .collect()
    }
    if KEEP_SPANS {
        return ty.clone();
    }
    syn::parse2(walk(ty.to_token_stream(), span)).unwrap_or_else(|_| ty.clone())
}

fn split_settings(tokens: TokenStream) -> Vec<TokenStream> {
    let mut settings = vec![TokenStream::new()];
    let mut depth = 0i32;
    for tree in tokens {
        match &tree {
            TokenTree::Punct(p) if p.as_char() == '<' => depth += 1,
            TokenTree::Punct(p) if p.as_char() == '>' => depth -= 1,
            TokenTree::Punct(p) if p.as_char() == ',' && depth == 0 => {
                settings.push(TokenStream::new());
                continue;
            }
            _ => {}
        }
        if let Some(last) = settings.last_mut() {
            last.extend([tree]);
        }
    }
    settings.into_iter().filter(|s| !s.is_empty()).collect()
}

fn join_settings(settings: impl Iterator<Item = TokenStream>) -> TokenStream {
    let mut out = TokenStream::new();
    for (index, setting) in settings.enumerate() {
        if index > 0 {
            out.extend(quote::quote!(,));
        }
        out.extend(setting);
    }
    out
}

fn implementation_of(setting: &TokenStream) -> Option<syn::Type> {
    let mut trees = setting.clone().into_iter();
    match (trees.next(), trees.next()) {
        (Some(TokenTree::Ident(word)), Some(TokenTree::Punct(eq)))
            if word == "implementation" && eq.as_char() == '=' =>
        {
            syn::parse2(trees.collect()).ok()
        }
        _ => None,
    }
}

fn dep_names(setting: &TokenStream) -> Option<Vec<String>> {
    let mut trees = setting.clone().into_iter();
    match (trees.next(), trees.next(), trees.next()) {
        (Some(TokenTree::Ident(word)), Some(TokenTree::Punct(_)), Some(TokenTree::Group(list)))
            if word == "deps" =>
        {
            Some(
                split_settings(list.stream())
                    .into_iter()
                    .filter_map(|entry| entry.into_iter().filter_map(|t| match t {
                        TokenTree::Ident(i) => Some(i.to_string()),
                        _ => None,
                    }).last())
                    .collect(),
            )
        }
        _ => None,
    }
}
const KEEP_SPANS: bool = false;

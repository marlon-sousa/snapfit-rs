//! Adapter: writes one method of a dispatch type's impl, a `match` that calls
//! the method on the implementation its variant holds. Plays the role of its
//! parent.

use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use super::target::{Target, Variant};
use super::trait_path::TraitPath;
use crate::adapters::syntax::conditional_part;
use crate::domain::{Execution, Rebuilding, Safety};

/// How the plan says each arm calls the implementation.
#[derive(Debug, Clone, Copy)]
pub(super) struct Forwarding {
    pub(super) rebuilding: Rebuilding,
    pub(super) execution: Execution,
    pub(super) safety: Safety,
}

pub(super) fn forwarded_method(
    method: &syn::TraitItemFn,
    forwarding: Forwarding,
    target: Target,
    path: &TraitPath,
) -> TokenStream {
    let mut signature = method.sig.clone();
    signature.inputs = method
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(plain_input)
        .collect();
    let arguments: Vec<_> = signature.inputs.iter().filter_map(binding).collect();
    let call = Call {
        signature: &signature,
        arguments: &arguments,
        path,
        forwarding,
    };
    let arms = target
        .variants()
        .into_iter()
        .map(|variant| call.arm(variant));
    let cfgs = method
        .attrs
        .iter()
        .filter_map(|attr| conditional_part(&attr.meta));
    quote! {
        #(#[#cfgs])*
        #[inline]
        #signature {
            match self {
                #(#arms)*
            }
        }
    }
}

/// The input as the impl's signature writes it: an argument bound to a plain
/// name it is passed on by, and `mut` dropped from a receiver and from a
/// binding, which the forwarding body never needs.
fn plain_input((position, input): (usize, &syn::FnArg)) -> syn::FnArg {
    match input {
        syn::FnArg::Receiver(receiver) if receiver.reference.is_none() => {
            syn::FnArg::Receiver(syn::Receiver {
                mutability: None,
                ..receiver.clone()
            })
        }
        syn::FnArg::Receiver(receiver) => syn::FnArg::Receiver(receiver.clone()),
        syn::FnArg::Typed(argument) => {
            let ident = match argument.pat.as_ref() {
                syn::Pat::Ident(binding) if binding.subpat.is_none() => binding.ident.clone(),
                _ => syn::Ident::new(
                    &format!("__snapfit_argument_{position}"),
                    Span::mixed_site(),
                ),
            };
            syn::FnArg::Typed(syn::PatType {
                pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                    attrs: Vec::new(),
                    by_ref: None,
                    mutability: None,
                    ident,
                    subpat: None,
                })),
                ..argument.clone()
            })
        }
    }
}

fn binding(input: &syn::FnArg) -> Option<&syn::Ident> {
    match input {
        syn::FnArg::Typed(argument) => match argument.pat.as_ref() {
            syn::Pat::Ident(binding) => Some(&binding.ident),
            _ => None,
        },
        syn::FnArg::Receiver(_) => None,
    }
}

/// What every arm of one method's `match` calls.
struct Call<'a> {
    signature: &'a syn::Signature,
    arguments: &'a [&'a syn::Ident],
    path: &'a TraitPath,
    forwarding: Forwarding,
}

impl Call<'_> {
    fn arm(&self, variant: Variant) -> TokenStream {
        let inner = syn::Ident::new("__snapfit_inner", Span::mixed_site());
        let implementation = &variant.implementation;
        let trait_path = self.path.plain();
        let method = &self.signature.ident;
        let turbofish = self.turbofish();
        let arguments = iter::once(&inner).chain(self.arguments.iter().copied());
        let awaited = match self.forwarding.execution {
            Execution::Sync => None,
            Execution::Async => Some(quote!(.await)),
        };
        let mut call = quote! {
            <#implementation as #trait_path>::#method #turbofish(#(#arguments),*) #awaited
        };
        if self.forwarding.safety.is_unsafe() {
            call = quote!(unsafe { #call });
        }
        let constructor = &variant.constructor;
        let value = match self.forwarding.rebuilding {
            Rebuilding::None => call,
            Rebuilding::IntoVariant => quote!(#constructor(#call)),
        };
        quote!(#constructor(#inner) => #value,)
    }

    /// `::<T, N>` for a method's own type and const parameters, so that one
    /// that no argument names is still passed on; lifetimes are left to
    /// inference, since a late-bound one cannot be written.
    fn turbofish(&self) -> Option<TokenStream> {
        let parameters: Vec<_> = self
            .signature
            .generics
            .params
            .iter()
            .filter_map(type_or_const)
            .collect();
        (!parameters.is_empty()).then(|| quote!(::<#(#parameters),*>))
    }
}

fn type_or_const(param: &syn::GenericParam) -> Option<&syn::Ident> {
    match param {
        syn::GenericParam::Type(param) => Some(&param.ident),
        syn::GenericParam::Const(param) => Some(&param.ident),
        syn::GenericParam::Lifetime(_) => None,
    }
}

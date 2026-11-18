//! Adapter: the expression that builds one dependency, from its step and its field's syntax.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;

use super::factories::{inputs_ident, method_ident, trait_ident};
use crate::adapters::syntax::{ContainerSyntax, FieldSyntax};
use crate::domain::{BuilderMethods, Call, Constructor, FunctionName, Step, Wrapping};

#[cfg(test)]
mod tests;

/// Where already-built singletons are read from while building a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValueSource {
    /// Local `let` bindings inside `new()` / `init()`.
    Locals,
    /// The container's own fields inside a getter.
    SelfFields,
}

/// Where a value is built: the container it belongs to, and where its
/// prerequisites are read from.
#[derive(Debug, Clone, Copy)]
pub(super) struct Site<'a> {
    pub(super) container: &'a ContainerSyntax,
    pub(super) source: ValueSource,
}

pub(super) fn construct(step: &Step<FieldSyntax>, site: Site<'_>) -> TokenStream {
    let args = step
        .prerequisites
        .iter()
        .map(|prerequisite| argument(prerequisite, site));
    let field = &step.syntax;
    let (call, wrapping) = match &step.call {
        Call::Constructor {
            constructor,
            wrapping,
        } => (
            constructor_call(target(field, *wrapping), constructor, field, site, args),
            *wrapping,
        ),
        Call::Builder { methods, wrapping } => (
            builder_call(target(field, *wrapping), methods, field, args),
            *wrapping,
        ),
        Call::Factory => (factory_call(step, site, args), Wrapping::None),
    };
    let value = if step.execution.is_async() {
        quote!(#call.await?)
    } else {
        call
    };
    match wrapping {
        Wrapping::Arc => quote!(::std::sync::Arc::new(#value)),
        Wrapping::None => value,
    }
}

/// A singleton is cloned from where it already lives; a transient is built
/// fresh for every argument it is passed to.
fn argument(prerequisite: &Step<FieldSyntax>, site: Site<'_>) -> TokenStream {
    if prerequisite.lifecycle.is_transient() {
        return construct(prerequisite, site);
    }
    let ident = &prerequisite.syntax.ident;
    match site.source {
        ValueSource::Locals => {
            let local = local_ident(ident);
            quote!(#local.clone())
        }
        ValueSource::SelfFields => quote!(self.#ident.clone()),
    }
}

/// `<Self as AppFactories>::build_auth(AppAuthInputs { config: ..., db: ... })`, at
/// the field, where a missing implementation of the trait is reported.
fn factory_call(
    step: &Step<FieldSyntax>,
    site: Site<'_>,
    args: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    let field = &step.syntax.ident;
    let factories_trait = trait_ident(&site.container.ident);
    let method = method_ident(field);
    let inputs = inputs_ident(&site.container.ident, field);
    let names = step
        .prerequisites
        .iter()
        .map(|prerequisite| &prerequisite.syntax.ident);
    quote_spanned!(field.span()=> <Self as #factories_trait>::#method(#inputs { #(#names: #args),* }))
}

/// The type whose function builds the value: `T` when the field holds `Arc<T>`.
fn target(field: &FieldSyntax, wrapping: Wrapping) -> &syn::Type {
    match wrapping {
        Wrapping::Arc => &field.arc_inner,
        Wrapping::None => &field.ty,
    }
}

/// `<T>::builder().disk(...).memory(...).build()`, each name with the user's
/// own token: the setter's label or `deps` entry, and the start and finish of
/// `builder(...)`, or the `builder` word for the ones left out. The chain
/// itself is spanned at the finish, where a typestate builder reports a
/// setter left out.
fn builder_call(
    target: &syn::Type,
    methods: &BuilderMethods,
    field: &FieldSyntax,
    args: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    let (start, finish) = match &field.builder {
        Some(builder) => (builder.start.clone(), builder.finish.clone()),
        None => (function_ident(&methods.start), function_ident(&methods.finish)),
    };
    let setters = &field.setters;
    quote_spanned!(finish.span()=> <#target>::#start()#(.#setters(#args))*.#finish())
}

/// `<T>::new(...)`, `<T as Default>::default()`, or `<T>::connect(...)` with
/// the user's own `connect`, from the field's `build` or the container's
/// `leaf_build`.
fn constructor_call(
    target: &syn::Type,
    constructor: &Constructor,
    field: &FieldSyntax,
    site: Site<'_>,
    args: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    match constructor {
        Constructor::New => quote!(<#target>::new(#(#args),*)),
        Constructor::Default => quote!(<#target as ::std::default::Default>::default()),
        Constructor::Named(function) => {
            let function = field
                .constructor
                .as_ref()
                .or(site.container.leaf_constructor.as_ref())
                .cloned()
                .unwrap_or_else(|| function_ident(function));
            quote!(<#target>::#function(#(#args),*))
        }
    }
}

fn function_ident(function: &FunctionName) -> syn::Ident {
    format_ident!("{}", function.as_str())
}

/// The private local a singleton is bound to inside `new()` / `init()`. A
/// prefix keeps it from clashing with anything the user has in scope, such as
/// a unit struct with the same name as a non-snake-case field.
pub(super) fn local_ident(field: &syn::Ident) -> syn::Ident {
    format_ident!("__snapfit_{}", field.unraw(), span = field.span())
}

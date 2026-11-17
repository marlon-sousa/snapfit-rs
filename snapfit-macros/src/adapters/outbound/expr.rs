//! Adapter: the expression that builds one dependency, from its step and its field's syntax.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

use crate::adapters::syntax::FieldSyntax;
use crate::domain::{Call, Constructor, FunctionName, Step, Wrapping};

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

pub(super) fn construct(step: &Step<FieldSyntax>, source: ValueSource) -> TokenStream {
    let args = step
        .prerequisites
        .iter()
        .map(|prerequisite| argument(prerequisite, source));
    let field = &step.syntax;
    let (call, wrapping) = match &step.call {
        Call::Constructor {
            constructor,
            wrapping,
        } => {
            let function = constructor_ident(constructor);
            let target = match wrapping {
                Wrapping::Arc => &field.arc_inner,
                Wrapping::None => &field.ty,
            };
            (quote!(<#target>::#function(#(#args),*)), *wrapping)
        }
        Call::Builder(function) => {
            let function = function_ident(function);
            (quote!(Self::#function(#(#args),*)), Wrapping::None)
        }
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
fn argument(prerequisite: &Step<FieldSyntax>, source: ValueSource) -> TokenStream {
    if prerequisite.lifecycle.is_transient() {
        return construct(prerequisite, source);
    }
    let ident = &prerequisite.syntax.ident;
    match source {
        ValueSource::Locals => {
            let local = local_ident(ident);
            quote!(#local.clone())
        }
        ValueSource::SelfFields => quote!(self.#ident.clone()),
    }
}

fn constructor_ident(constructor: &Constructor) -> syn::Ident {
    match constructor {
        Constructor::New => format_ident!("new"),
        Constructor::Named(function) => function_ident(function),
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

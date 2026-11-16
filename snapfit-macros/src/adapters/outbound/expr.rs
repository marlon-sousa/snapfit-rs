use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

use crate::domain::{Construction, ConstructionPlan, DependencyName, InstantiationStep};
use crate::ports::{ContainerSyntax, FieldSyntax};

#[cfg(test)]
mod tests;

/// Where already-built singletons are read from while constructing a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValueSource {
    /// Local `let` bindings inside `new()` / `init()`.
    Locals,
    /// The container's own fields inside a getter.
    SelfFields,
}

/// Builds the expression that constructs one dependency, entirely from typed
/// syntax: `<T>::ctor(args)` never needs to re-parse generated text.
pub(super) struct ExprBuilder<'a> {
    plan: &'a ConstructionPlan,
    syntax: &'a ContainerSyntax,
    source: ValueSource,
}

impl<'a> ExprBuilder<'a> {
    pub(super) fn new(
        plan: &'a ConstructionPlan,
        syntax: &'a ContainerSyntax,
        source: ValueSource,
    ) -> Self {
        Self {
            plan,
            syntax,
            source,
        }
    }

    pub(super) fn construct(&self, step: &InstantiationStep) -> TokenStream {
        let field = self.field(&step.dep_name);
        let args = step
            .prerequisites
            .iter()
            .map(|prereq| self.argument(prereq));
        let call = match &step.construction {
            Construction::Constructor(method) => {
                let method = format_ident!("{method}");
                let target = arc_inner_type(&field.ty).unwrap_or(&field.ty);
                quote!(<#target>::#method(#(#args),*))
            }
            Construction::Builder(function) => {
                let function = format_ident!("{function}");
                quote!(Self::#function(#(#args),*))
            }
        };
        let value = if step.is_async {
            quote!(#call.await?)
        } else {
            call
        };
        let wraps_in_arc = matches!(step.construction, Construction::Constructor(_))
            && arc_inner_type(&field.ty).is_some();
        if wraps_in_arc {
            quote!(::std::sync::Arc::new(#value))
        } else {
            value
        }
    }

    /// A singleton is cloned from where it already lives; a transient is
    /// built fresh for every argument it is passed to.
    fn argument(&self, prereq: &DependencyName) -> TokenStream {
        let step = self
            .plan
            .step(prereq)
            .expect("the graph engine guarantees every prerequisite has a step");
        if step.lifecycle.is_transient() {
            return self.construct(step);
        }
        let ident = &self.field(prereq).ident;
        match self.source {
            ValueSource::Locals => {
                let local = local_ident(ident);
                quote!(#local.clone())
            }
            ValueSource::SelfFields => quote!(self.#ident.clone()),
        }
    }

    fn field(&self, name: &DependencyName) -> &FieldSyntax {
        self.syntax
            .field(name.as_str())
            .expect("every planned dependency comes from a named container field")
    }
}

/// The private local a singleton is bound to inside `new()` / `init()`. A
/// prefix keeps it from clashing with anything the user has in scope, such as
/// a unit struct with the same name as a non-snake-case field.
pub(super) fn local_ident(field: &syn::Ident) -> syn::Ident {
    format_ident!("__snapfit_{}", field.unraw(), span = field.span())
}

/// `T` when `ty` is `Arc<T>`, `std::sync::Arc<T>`, `::std::sync::Arc<T>` or
/// `alloc::sync::Arc<T>`.
pub(super) fn arc_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    if type_path.qself.is_some() {
        return None;
    }
    let segments: Vec<_> = type_path.path.segments.iter().collect();
    let last = segments.last()?;
    let is_arc_path = last.ident == "Arc"
        && match segments.as_slice() {
            [_] => type_path.path.leading_colon.is_none(),
            [.., sync, _] => sync.ident == "sync",
            _ => false,
        };
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    match (is_arc_path, args.args.first(), args.args.len()) {
        (true, Some(syn::GenericArgument::Type(inner)), 1) => Some(inner),
        _ => None,
    }
}

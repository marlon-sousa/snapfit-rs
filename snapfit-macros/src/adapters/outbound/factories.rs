//! Adapter: the factories trait and one inputs struct per factory, when any field has `build = factory`.

use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

use crate::adapters::syntax::{FieldSyntax, Plan};
use crate::domain::Step;

#[cfg(test)]
mod tests;

pub(super) fn emit_factories(plan: &Plan) -> TokenStream {
    let factories: Vec<_> = plan
        .declaration_order
        .iter()
        .filter(|step| step.call.is_factory())
        .collect();
    if factories.is_empty() {
        return TokenStream::new();
    }
    let container = &plan.syntax.ident;
    let vis = &plan.syntax.vis;
    let factories_trait = trait_ident(container);
    let trait_doc = format!(
        "The factories of `{container}`, one for each field marked `build = factory`, which you implement on `{container}`."
    );
    let methods = factories.iter().map(|step| method(container, step));
    let inputs = factories
        .iter()
        .map(|step| inputs_struct(container, vis, step));

    quote! {
        #[doc = #trait_doc]
        #vis trait #factories_trait {
            #(#methods)*
        }

        #(#inputs)*
    }
}

/// `<Container>Factories`.
pub(super) fn trait_ident(container: &syn::Ident) -> syn::Ident {
    format_ident!("{}Factories", container.unraw())
}

/// `build_<field>`.
pub(super) fn method_ident(field: &syn::Ident) -> syn::Ident {
    format_ident!("build_{}", field.unraw())
}

/// `<Container><Field>Inputs`, the field's name in PascalCase.
pub(super) fn inputs_ident(container: &syn::Ident, field: &syn::Ident) -> syn::Ident {
    let field = field.unraw().to_string().to_upper_camel_case();
    format_ident!("{}{}Inputs", container.unraw(), field)
}

/// `fn build_<field>(inputs: <Container><Field>Inputs) -> <field's type>;`
fn method(container: &syn::Ident, step: &Step<FieldSyntax>) -> TokenStream {
    let FieldSyntax { ident, ty, .. } = &step.syntax;
    let method = method_ident(ident);
    let inputs = inputs_ident(container, ident);
    let doc = format!("Builds `{}` from its inputs.", ident.unraw());
    quote! {
        #[doc = #doc]
        fn #method(inputs: #inputs) -> #ty;
    }
}

/// One `pub` field for each prerequisite, with its name and its field's type.
fn inputs_struct(
    container: &syn::Ident,
    vis: &syn::Visibility,
    step: &Step<FieldSyntax>,
) -> TokenStream {
    let field = &step.syntax.ident;
    let inputs = inputs_ident(container, field);
    let doc = format!(
        "The dependencies `{}` is built from, by name, passed to `{}::{}`.",
        field.unraw(),
        trait_ident(container),
        method_ident(field)
    );
    let fields = step
        .prerequisites
        .iter()
        .map(|prerequisite| input_field(&prerequisite.syntax));
    quote! {
        #[doc = #doc]
        #vis struct #inputs {
            #(#fields,)*
        }
    }
}

fn input_field(prerequisite: &FieldSyntax) -> TokenStream {
    let FieldSyntax { ident, ty, .. } = prerequisite;
    let doc = format!("The container's `{}`.", ident.unraw());
    quote! {
        #[doc = #doc]
        pub #ident: #ty
    }
}

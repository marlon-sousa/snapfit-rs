use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::step_expr::{build_call_expr, parse_type_tokens, references_dependency};
use crate::domain::{ConstructionPlan, InstantiationStep};

#[cfg(test)]
mod tests;

pub(super) fn emit_getters_impl(plan: &ConstructionPlan) -> TokenStream {
    let container_ident = format_ident!("{}", plan.container_name.as_str());
    let direct_getters: Vec<_> = plan
        .ordered_steps
        .iter()
        .map(|step| emit_direct_getter(plan, step))
        .collect();

    quote! {
        impl #container_ident {
            #(#direct_getters)*
        }
    }
}

fn emit_direct_getter(plan: &ConstructionPlan, step: &InstantiationStep) -> TokenStream {
    let getter_ident = getter_ident(step);
    let dep_type = parse_type_tokens(step.dep_type.as_str());

    if step.lifecycle.is_singleton() {
        let field_ident = format_ident!("{}", step.dep_name.as_str());
        return quote! {
            pub fn #getter_ident(&self) -> #dep_type {
                self.#field_ident.clone()
            }
        };
    }

    let prereq_bindings = emit_transient_prereq_bindings(plan, step);
    let construct_expr = build_call_expr(plan, step);
    quote! {
        pub fn #getter_ident(&self) -> #dep_type {
            #(#prereq_bindings)*
            #construct_expr
        }
    }
}

fn emit_transient_prereq_bindings(
    plan: &ConstructionPlan,
    step: &InstantiationStep,
) -> Vec<TokenStream> {
    plan.ordered_steps
        .iter()
        .filter(|candidate| {
            references_dependency(&step.constructor_call, candidate.dep_name.as_str())
        })
        .map(|prereq| {
            let prereq_ident = format_ident!("{}", prereq.dep_name.as_str());
            let prereq_getter = getter_ident(prereq);
            quote!(let #prereq_ident = self.#prereq_getter();)
        })
        .collect()
}

fn getter_ident(step: &InstantiationStep) -> syn::Ident {
    format_ident!("get_{}", step.dep_name.as_str())
}

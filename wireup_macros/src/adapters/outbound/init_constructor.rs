use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::step_expr::{
    build_call_expr, is_step_needed_at_init, parse_expr_tokens, parse_type_tokens,
    wrap_in_arc_if_needed,
};
use crate::domain::{ConstructionPlan, DependencyName, InstantiationStep};

#[cfg(test)]
mod tests;

pub(super) fn emit_struct_and_init(plan: &ConstructionPlan) -> TokenStream {
    let struct_def = emit_struct_definition(plan);
    let init_impl = if plan.is_async {
        emit_async_init_impl(plan)
    } else {
        emit_sync_new_impl(plan)
    };

    quote! {
        #struct_def
        #init_impl
    }
}

fn emit_struct_definition(plan: &ConstructionPlan) -> TokenStream {
    let container_ident = format_ident!("{}", plan.container_name.as_str());
    let fields: Vec<_> = plan
        .ordered_steps
        .iter()
        .filter(|step| step.lifecycle.is_singleton())
        .map(emit_struct_field)
        .collect();

    quote! {
        pub struct #container_ident {
            #(#fields,)*
        }
    }
}

fn emit_struct_field(step: &InstantiationStep) -> TokenStream {
    let field_ident = format_ident!("{}", step.dep_name.as_str());
    let field_type = parse_type_tokens(step.dep_type.as_str());
    quote!(#field_ident: #field_type)
}

fn emit_sync_new_impl(plan: &ConstructionPlan) -> TokenStream {
    let container_ident = format_ident!("{}", plan.container_name.as_str());
    let statements = emit_sequential_statements(plan);
    let field_idents = singleton_field_idents(plan);

    quote! {
        impl #container_ident {
            pub fn new() -> Self {
                #(#statements)*
                Self {
                    #(#field_idents,)*
                }
            }
        }

        impl ::wireup::Container for #container_ident {
            fn new() -> Self {
                Self::new()
            }
        }
    }
}

fn emit_async_init_impl(plan: &ConstructionPlan) -> TokenStream {
    let container_ident = format_ident!("{}", plan.container_name.as_str());
    let statements = emit_async_statements(plan);
    let field_idents = singleton_field_idents(plan);

    quote! {
        impl #container_ident {
            pub async fn init() -> ::std::result::Result<
                Self,
                ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync>,
            > {
                #(#statements)*
                ::std::result::Result::Ok(Self {
                    #(#field_idents,)*
                })
            }
        }
    }
}

fn emit_sequential_statements(plan: &ConstructionPlan) -> Vec<TokenStream> {
    plan.ordered_steps
        .iter()
        .filter(|step| is_step_needed_at_init(step, &plan.ordered_steps))
        .map(|step| emit_step_let_binding(plan, step))
        .collect()
}

fn emit_async_statements(plan: &ConstructionPlan) -> Vec<TokenStream> {
    let mut emitted = HashSet::new();
    let mut statements = Vec::new();

    for step in plan
        .ordered_steps
        .iter()
        .filter(|s| is_step_needed_at_init(s, &plan.ordered_steps))
    {
        if !emitted.insert(step.dep_name.clone()) {
            continue;
        }
        if let Some(batch) = find_parallel_batch(plan, &step.dep_name) {
            batch.iter().for_each(|dep| {
                emitted.insert(dep.clone());
            });
            statements.push(emit_parallel_batch(plan, batch));
        } else {
            statements.push(emit_step_let_binding(plan, step));
        }
    }

    statements
}

fn find_parallel_batch<'a>(
    plan: &'a ConstructionPlan,
    dep_name: &DependencyName,
) -> Option<&'a [DependencyName]> {
    plan.parallel_batches
        .iter()
        .find(|batch| batch.len() > 1 && batch.contains(dep_name))
        .map(Vec::as_slice)
}

fn emit_parallel_batch(plan: &ConstructionPlan, batch: &[DependencyName]) -> TokenStream {
    let batch_steps: Vec<_> = batch
        .iter()
        .filter_map(|name| plan.ordered_steps.iter().find(|s| &s.dep_name == name))
        .collect();
    let res_idents: Vec<_> = batch_steps
        .iter()
        .map(|s| format_ident!("{}_res", s.dep_name.as_str()))
        .collect();
    let call_exprs: Vec<_> = batch_steps
        .iter()
        .map(|s| parse_expr_tokens(&s.constructor_call))
        .collect();
    let bindings = emit_batch_unwrapping(plan, &batch_steps, &res_idents);

    quote! {
        let (#(#res_idents),*) = ::tokio::join!(#(#call_exprs),*);
        #(#bindings)*
    }
}

fn emit_batch_unwrapping(
    plan: &ConstructionPlan,
    batch_steps: &[&InstantiationStep],
    res_idents: &[syn::Ident],
) -> Vec<TokenStream> {
    batch_steps
        .iter()
        .zip(res_idents)
        .map(|(step, res_ident)| {
            let dep_ident = format_ident!("{}", step.dep_name.as_str());
            let dep_type = parse_type_tokens(step.dep_type.as_str());
            let value_expr = wrap_in_arc_if_needed(plan, step, quote!(#res_ident?));
            quote!(let #dep_ident: #dep_type = #value_expr;)
        })
        .collect()
}

fn emit_step_let_binding(plan: &ConstructionPlan, step: &InstantiationStep) -> TokenStream {
    let dep_ident = format_ident!("{}", step.dep_name.as_str());
    let dep_type = parse_type_tokens(step.dep_type.as_str());
    let init_expr = build_call_expr(plan, step);
    quote!(let #dep_ident: #dep_type = #init_expr;)
}

fn singleton_field_idents(plan: &ConstructionPlan) -> Vec<syn::Ident> {
    plan.ordered_steps
        .iter()
        .filter(|step| step.lifecycle.is_singleton())
        .map(|step| format_ident!("{}", step.dep_name.as_str()))
        .collect()
}

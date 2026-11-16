use proc_macro2::TokenStream;
use quote::quote;

use crate::domain::{ConstructionPlan, InstantiationStep};

#[cfg(test)]
mod tests;

pub(super) fn build_call_expr(plan: &ConstructionPlan, step: &InstantiationStep) -> TokenStream {
    let raw_call = parse_expr_tokens(&step.constructor_call);
    let awaited = if step.is_async {
        quote!(#raw_call.await?)
    } else {
        raw_call
    };
    wrap_in_arc_if_needed(plan, step, awaited)
}

pub(super) fn wrap_in_arc_if_needed(
    plan: &ConstructionPlan,
    step: &InstantiationStep,
    inner_expr: TokenStream,
) -> TokenStream {
    let trimmed = step.dep_type.as_str().trim();
    let normalized = trimmed.strip_prefix("::").unwrap_or(trimmed);
    let is_arc_type = ["std::sync::Arc<", "alloc::sync::Arc<", "Arc<"]
        .into_iter()
        .any(|prefix| normalized.starts_with(prefix) && normalized.ends_with('>'));
    let is_custom_builder = step
        .constructor_call
        .starts_with(&format!("{}::", plan.container_name));

    if is_arc_type && !is_custom_builder {
        quote!(::std::sync::Arc::new(#inner_expr))
    } else {
        inner_expr
    }
}

pub(super) fn is_step_needed_at_init(
    step: &InstantiationStep,
    all_steps: &[InstantiationStep],
) -> bool {
    step.lifecycle.is_singleton()
        || all_steps
            .iter()
            .filter(|s| s.lifecycle.is_singleton())
            .any(|s| references_dependency(&s.constructor_call, step.dep_name.as_str()))
}

pub(super) fn references_dependency(constructor_call: &str, dep_name: &str) -> bool {
    let Some((_, rest)) = constructor_call.split_once('(') else {
        return false;
    };
    let Some((args_str, _)) = rest.rsplit_once(')') else {
        return false;
    };
    args_str
        .split(',')
        .filter_map(|arg| arg.trim().strip_suffix(".clone()"))
        .any(|ident| ident.trim() == dep_name)
}

pub(super) fn parse_type_tokens(type_str: &str) -> TokenStream {
    type_str
        .parse()
        .expect("TypeName must contain valid Rust type tokens")
}

pub(super) fn parse_expr_tokens(expr_str: &str) -> TokenStream {
    expr_str
        .parse()
        .expect("constructor_call must contain valid Rust expression tokens")
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::key_enum::variant_ident;
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
    let get_dependency_fn = emit_get_dependency_fn(plan);
    let downcast_fn = emit_downcast_fn();

    quote! {
        impl #container_ident {
            #(#direct_getters)*
            #get_dependency_fn
            #downcast_fn
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

fn emit_get_dependency_fn(plan: &ConstructionPlan) -> TokenStream {
    let key_enum_ident = format_ident!("{}", plan.key_enum_name);
    let match_arms: Vec<_> = plan
        .ordered_steps
        .iter()
        .map(|step| emit_key_match_arm(&key_enum_ident, step))
        .collect();

    quote! {
        pub fn get_dependency<T: 'static + Clone>(&self, key: impl ::std::convert::AsRef<str>) -> T {
            let key_str = key.as_ref();
            let enum_key: #key_enum_ident = key_str.parse().unwrap_or_else(|_| {
                panic!("[wireup Error] Unknown dependency key: \"{}\"", key_str);
            });

            match enum_key {
                #(#match_arms,)*
            }
        }
    }
}

fn emit_key_match_arm(key_enum_ident: &syn::Ident, step: &InstantiationStep) -> TokenStream {
    let variant = variant_ident(step);
    let dep_type = parse_type_tokens(step.dep_type.as_str());
    let getter = getter_ident(step);

    quote! {
        #key_enum_ident::#variant => self.downcast_dep::<#dep_type, T>(self.#getter(), key_str)
    }
}

fn emit_downcast_fn() -> TokenStream {
    quote! {
        #[allow(dead_code)]
        fn downcast_dep<Actual: 'static + Clone, Target: 'static + Clone>(
            &self,
            val: Actual,
            key: &str,
        ) -> Target {
            let any_val = &val as &dyn ::std::any::Any;
            match any_val.downcast_ref::<Target>() {
                ::std::option::Option::Some(matched) => matched.clone(),
                ::std::option::Option::None => {
                    panic!(
                        "\n[wireup Error] Type mismatch for dependency key: \"{}\"\n\
                         Expected type: {}\n\
                         Actual type:   {}\n",
                        key,
                        ::std::any::type_name::<Target>(),
                        ::std::any::type_name::<Actual>()
                    );
                }
            }
        }
    }
}

fn getter_ident(step: &InstantiationStep) -> syn::Ident {
    format_ident!("get_{}", step.dep_name.as_str())
}

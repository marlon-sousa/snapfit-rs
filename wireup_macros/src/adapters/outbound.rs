mod getters;
mod init_constructor;
mod key_enum;
mod step_expr;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::quote;

use crate::domain::ConstructionPlan;
use crate::ports::CodeEmitterPort;

#[derive(Debug, Default, Clone)]
pub(crate) struct QuoteEmitterAdapter;

impl QuoteEmitterAdapter {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl CodeEmitterPort for QuoteEmitterAdapter {
    fn emit_container(&self, plan: &ConstructionPlan) -> TokenStream {
        let struct_and_init = init_constructor::emit_struct_and_init(plan);
        let key_enum = key_enum::emit_key_enum(plan);
        let getters = getters::emit_getters_impl(plan);

        quote! {
            #struct_and_init
            #key_enum
            #getters
        }
    }
}

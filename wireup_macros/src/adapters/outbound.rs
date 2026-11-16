mod getters;
mod init_constructor;
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
        let getters = getters::emit_getters_impl(plan);

        quote! {
            #struct_and_init
            #getters
        }
    }
}

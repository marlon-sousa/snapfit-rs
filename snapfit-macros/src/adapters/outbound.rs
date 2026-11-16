mod expr;
mod getters;
mod init_constructor;
mod struct_def;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use derive_new::new;
use proc_macro2::TokenStream;
use quote::quote;

use crate::domain::ConstructionPlan;
use crate::ports::{CodeEmitterPort, ContainerSyntax};

#[derive(new, Debug, Default, Clone)]
pub(crate) struct QuoteEmitterAdapter;

impl QuoteEmitterAdapter {
    /// `#[derive(Container)]` cannot rewrite the struct, so it only adds impls.
    pub(crate) fn emit_derive_impls(
        &self,
        plan: &ConstructionPlan,
        syntax: &ContainerSyntax,
    ) -> TokenStream {
        let init_impl = init_constructor::emit_init_impl(plan, syntax);
        let getters = getters::emit_getters_impl(plan, syntax);

        quote! {
            #init_impl
            #getters
        }
    }
}

impl CodeEmitterPort for QuoteEmitterAdapter {
    fn emit_container(&self, plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream {
        let struct_def = struct_def::emit_struct_definition(plan, syntax);
        let impls = self.emit_derive_impls(plan, syntax);

        quote! {
            #struct_def
            #impls
        }
    }
}

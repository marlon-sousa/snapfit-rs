use crate::domain::ConstructionPlan;
use proc_macro2::TokenStream;

pub trait CodeEmitterPort {
    fn emit_container(&self, plan: &ConstructionPlan) -> TokenStream;
}

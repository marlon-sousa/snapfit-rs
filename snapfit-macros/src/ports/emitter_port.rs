use crate::domain::ConstructionPlan;
use crate::ports::ContainerSyntax;
use proc_macro2::TokenStream;

pub trait CodeEmitterPort {
    fn emit_container(&self, plan: &ConstructionPlan, syntax: &ContainerSyntax) -> TokenStream;
}

//! Test support: plans built by the real parser and service, as the macro builds them.

use crate::adapters::inbound::parse_attribute;
use crate::adapters::syntax::{FieldSyntax, Plan};
use crate::domain::{FeatureGate, Step};
use crate::use_cases::plan_container;

pub(super) fn plan_for(item: syn::ItemStruct) -> Plan {
    let declaration = parse_attribute(proc_macro2::TokenStream::new(), &item).unwrap();
    plan_container(declaration, FeatureGate::Unstable).unwrap()
}

pub(super) fn step<'a>(plan: &'a Plan, name: &str) -> &'a Step<FieldSyntax> {
    plan.declaration_order
        .iter()
        .find(|step| step.name.as_str() == name)
        .unwrap()
}

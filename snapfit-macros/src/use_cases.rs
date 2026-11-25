//! Service: the domain's decisions about a declared container or dispatched
//! trait, and the services that run them.

mod declaration_rules;
mod derive_container;
mod dispatch_rules;
mod dispatch_trait;
mod execution;
mod fallibility;
mod feature_gate;
mod forwarding;
mod leaf;
mod repeats;
mod resolver;
mod static_dispatch;

pub(crate) use derive_container::{declared_dispatched_types, plan_container};
pub(crate) use dispatch_trait::plan_dispatch;

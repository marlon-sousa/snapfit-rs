//! Service: the domain's decisions about a declared container, and the service
//! that runs them.

mod declaration_rules;
mod derive_container;
mod execution;
mod feature_gate;
mod resolver;

pub(crate) use derive_container::plan_container;

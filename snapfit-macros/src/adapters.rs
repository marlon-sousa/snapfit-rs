//! Adapter: the two ends of the macro, reading the user's struct and writing
//! the generated code and diagnostics.

mod inbound;
mod outbound;
mod syntax;

pub(crate) use inbound::parse_attribute;
pub(crate) use outbound::{Origins, emit_container, fallback_struct};

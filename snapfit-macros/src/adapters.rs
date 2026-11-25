//! Adapter: the two ends of the macros, reading the user's struct or trait and
//! writing the generated code and diagnostics.

mod inbound;
mod outbound;
mod syntax;

pub(crate) use inbound::{parse_attribute, read_dispatch_attribute};
pub(crate) use outbound::{
    Origins, TraitOrigins, emit_aliases, emit_container, emit_dispatch_impls, fallback_struct,
};

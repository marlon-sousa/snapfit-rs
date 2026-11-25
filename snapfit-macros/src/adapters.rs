//! Adapter: the two ends of the macros, reading the user's struct or trait and
//! writing the generated code and diagnostics.

mod inbound;
mod outbound;
mod syntax;

pub(crate) use inbound::{
    parse_attribute, read_dispatch_attribute, read_readable_parts, readable_runtime_path,
};
pub(crate) use outbound::{
    Origins, TraitOrigins, emit_container, emit_dispatch_impls, emit_fallback_aliases,
    fallback_struct,
};

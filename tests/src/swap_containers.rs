//! Containers that swap the implementation an `Arc<dyn Trait>` field names,
//! while the services that take the trait object keep their code.

pub mod built_desks;
pub mod desks;
pub mod local;
pub mod readers;
#[cfg(feature = "unstable")]
pub mod transient_desk;
pub mod written_by_a_macro;

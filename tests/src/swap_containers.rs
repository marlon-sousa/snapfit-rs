//! Containers that swap the type of one field for another implementing the
//! same trait, while the services that depend on it keep their code: taking
//! the trait object of a field that names its `implementation`.

pub mod built_desks;
pub mod desks;
pub mod readers;
#[cfg(feature = "unstable")]
pub mod transient_desk;
pub mod written_by_a_macro;

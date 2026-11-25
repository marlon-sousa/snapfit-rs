//! Error: why a field's builder settings, labelled deps or setters cannot be
//! used.

use derive_more::Display;

use crate::domain::names::{DependencyName, SetterName};

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum BuilderProblem {
    #[display(
        "builder({written}) on field '{dependency}' cannot be read; builder takes start and finish, each with a function's name, such as builder(start = new_builder, finish = done)"
    )]
    InvalidSetting {
        dependency: DependencyName,
        written: String,
    },
    #[display(
        "the dependency '{label} = {field}' of field '{dependency}' has a label, which names a builder setter; write {field} alone, or build the field with build = builder"
    )]
    LabelWithoutBuilder {
        dependency: DependencyName,
        label: SetterName,
        field: DependencyName,
    },
    #[display(
        "field '{dependency}' passes more than one dependency to the builder setter '{setter}'; give each dependency its own setter"
    )]
    DuplicateSetter {
        dependency: DependencyName,
        setter: SetterName,
    },
}

impl BuilderProblem {
    pub(crate) fn subject(&self) -> &DependencyName {
        match self {
            Self::InvalidSetting { dependency, .. }
            | Self::LabelWithoutBuilder { dependency, .. }
            | Self::DuplicateSetter { dependency, .. } => dependency,
        }
    }
}

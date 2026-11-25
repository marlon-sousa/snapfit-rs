//! Error: why the dependencies the fields name cannot be wired into a graph.

use derive_more::Display;

use super::sentences::{cycle_sentence, dependencies_noun, join_names};
use crate::domain::names::DependencyName;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum GraphProblem {
    #[display("dependency cycle: {}", cycle_sentence(start, through))]
    Cycle {
        start: DependencyName,
        /// The dependencies between `start` and its return to `start`.
        through: Vec<DependencyName>,
    },
    #[display(
        "unknown {} '{}' on field '{dependency}'. You must choose between the dependencies the container can wire: [{}]",
        dependencies_noun(missing),
        join_names(missing),
        join_names(available)
    )]
    UnknownPrerequisites {
        dependency: DependencyName,
        missing: Vec<DependencyName>,
        available: Vec<DependencyName>,
    },
    #[display(
        "field '{dependency}' lists '{prerequisite}' in deps more than once; list each dependency once"
    )]
    RepeatedPrerequisite {
        dependency: DependencyName,
        prerequisite: DependencyName,
    },
}

impl GraphProblem {
    pub(crate) fn subject(&self) -> &DependencyName {
        match self {
            Self::Cycle { start, .. } => start,
            Self::UnknownPrerequisites { dependency, .. }
            | Self::RepeatedPrerequisite { dependency, .. } => dependency,
        }
    }
}

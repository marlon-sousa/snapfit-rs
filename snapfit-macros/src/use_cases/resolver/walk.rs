//! Policy: what the resolver's walk keeps about each dependency it meets.

use std::rc::Rc;

use crate::domain::{DependencyDeclaration, DependencyName, Prerequisite, Step};

/// What visiting one name found.
pub(super) enum Outcome<F> {
    Built(Rc<Step<F>>),
    /// No field of the container has this name.
    Unknown,
    /// Declared, but it or something it needs could not be built.
    Failed,
}

impl<F> Outcome<F> {
    pub(super) fn into_step(self) -> Option<Rc<Step<F>>> {
        match self {
            Self::Built(step) => Some(step),
            Self::Unknown | Self::Failed => None,
        }
    }
}

/// A declaration waiting to be built, and where its field is declared.
pub(super) struct Declared<F, T> {
    pub(super) position: usize,
    pub(super) declaration: DependencyDeclaration<F, T>,
}

/// A dependency being built, which the one after it on the path needs.
pub(super) struct OnPath {
    pub(super) position: usize,
    pub(super) name: DependencyName,
}

/// A built step, and where its field is declared.
pub(super) struct Placed<F> {
    pub(super) position: usize,
    pub(super) step: Rc<Step<F>>,
}

pub(super) fn is_unknown<F>(pair: &(&Prerequisite, &Outcome<F>)) -> bool {
    matches!(pair.1, Outcome::Unknown)
}

pub(super) fn name_of_outcome<F>(pair: (&Prerequisite, &Outcome<F>)) -> DependencyName {
    pair.0.field.clone()
}

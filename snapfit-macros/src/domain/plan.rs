//! Entity: how a container is built, step by step, as the emitter writes it.

use std::rc::Rc;

use derive_new::new;
use strum::EnumIs;

use crate::domain::declaration::{BuilderMethods, Constructor, Execution, Fallibility, Lifecycle};
use crate::domain::names::DependencyName;

#[cfg(test)]
mod tests;

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wrapping {
    None,
    /// The constructor builds `T`, and the field holds `Arc<T>`.
    Arc,
}

#[derive(EnumIs, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Call {
    Constructor {
        constructor: Constructor,
        wrapping: Wrapping,
    },
    /// The field's `build_<field>` method of the container's factories trait.
    Factory,
    /// The type's builder, with one setter per prerequisite.
    Builder {
        methods: BuilderMethods,
        wrapping: Wrapping,
    },
}

#[derive(new, Debug, PartialEq, Eq)]
pub(crate) struct Step<F> {
    pub(crate) name: DependencyName,
    pub(crate) syntax: F,
    pub(crate) lifecycle: Lifecycle,
    pub(crate) execution: Execution,
    pub(crate) fallibility: Fallibility,
    pub(crate) call: Call,
    /// The steps whose values are passed to the call, in the order `deps` lists them.
    pub(crate) prerequisites: Vec<Rc<Step<F>>>,
}

#[derive(new, Debug, PartialEq, Eq)]
pub(crate) struct ConstructionPlan<C, F> {
    pub(crate) syntax: C,
    pub(crate) execution: Execution,
    /// Whether building the container can fail, which a fallible step makes it.
    pub(crate) fallibility: Fallibility,
    /// Every step after the steps it needs.
    pub(crate) construction_order: Vec<Rc<Step<F>>>,
    /// The same steps, in the order the fields are declared.
    pub(crate) declaration_order: Vec<Rc<Step<F>>>,
}

impl<C, F> ConstructionPlan<C, F> {
    pub(crate) fn singletons(&self) -> impl Iterator<Item = &Step<F>> {
        self.construction_order
            .iter()
            .map(Rc::as_ref)
            .filter(is_singleton)
    }
}

fn is_singleton<F>(step: &&Step<F>) -> bool {
    step.lifecycle.is_singleton()
}

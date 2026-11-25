//! Policy: orders the declared dependencies so that each is built after the
//! ones it needs, links every step to its prerequisites' steps, and finds
//! every duplicate, unknown prerequisite and cycle on the way.

mod call;
mod cycle;
mod resolved_steps;
mod step;
#[cfg(test)]
mod tests;
mod walk;

use super::repeats::repeats;
pub(crate) use resolved_steps::ResolvedSteps;
use step::step_of;
use walk::{Declared, OnPath, Outcome, Placed, is_unknown, name_of_outcome};

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::domain::{
    Constructor, ContainerDeclaration, DependencyDeclaration, DependencyName, DomainError,
    DomainErrors, Step,
};
use crate::use_cases::static_dispatch::{DispatchedNames, dispatch_of};

/// Orders and links the steps, or reports every problem the graph of
/// dependencies has.
///
/// Dependencies are visited in declaration order, each after its
/// prerequisites, so the construction order follows the fields wherever the
/// graph allows it.
pub(crate) fn resolve<C, F, S, T>(
    declaration: ContainerDeclaration<C, F, S, T>,
) -> Result<ResolvedSteps<C, F>, DomainErrors> {
    let ContainerDeclaration {
        syntax,
        dependencies,
        leaf_constructor,
        dispatched_types,
        ..
    } = declaration;
    let declared = dispatched_types.into_iter().collect();
    let mut resolver = Resolver::new(dependencies, leaf_constructor, declared);
    for name in resolver.declared.clone() {
        resolver.visit(&name);
    }
    DomainErrors::check(resolver.errors)?;
    Ok(ResolvedSteps::new(syntax, resolver.construction_order))
}

struct Resolver<F, T> {
    /// Field names in declaration order, the first of any duplicates only.
    declared: Vec<DependencyName>,
    pending: HashMap<DependencyName, Declared<F, T>>,
    /// The dependencies being built, each needed by the one before it.
    path: Vec<OnPath>,
    built: HashMap<DependencyName, Rc<Step<F>>>,
    failed: HashSet<DependencyName>,
    construction_order: Vec<Placed<F>>,
    errors: Vec<DomainError>,
    /// How a field with no `deps` and no `build` of its own is built.
    leaf: Constructor,
    /// The names the container's `static_dispatch_for` declare.
    dispatched: DispatchedNames,
}

impl<F, T> Resolver<F, T> {
    fn new(
        dependencies: Vec<DependencyDeclaration<F, T>>,
        leaf: Constructor,
        dispatched: DispatchedNames,
    ) -> Self {
        let mut resolver = Self {
            declared: Vec::with_capacity(dependencies.len()),
            pending: HashMap::with_capacity(dependencies.len()),
            path: Vec::new(),
            built: HashMap::new(),
            failed: HashSet::new(),
            construction_order: Vec::with_capacity(dependencies.len()),
            errors: Vec::new(),
            leaf,
            dispatched,
        };
        for dependency in dependencies {
            resolver.declare(dependency);
        }
        resolver
    }

    fn declare(&mut self, declaration: DependencyDeclaration<F, T>) {
        if self.pending.contains_key(&declaration.name) {
            self.errors
                .push(DomainError::DuplicateDependency(declaration.name));
            return;
        }
        let position = self.declared.len();
        self.declared.push(declaration.name.clone());
        let name = declaration.name.clone();
        self.pending.insert(
            name,
            Declared {
                position,
                declaration,
            },
        );
    }

    fn visit(&mut self, name: &DependencyName) -> Outcome<F> {
        if let Some(step) = self.built.get(name) {
            return Outcome::Built(Rc::clone(step));
        }
        if self.failed.contains(name) {
            return Outcome::Failed;
        }
        if let Some(start) = self.path.iter().position(|on_path| &on_path.name == name) {
            self.errors
                .extend(cycle::cycle_through(self.path.iter().skip(start)));
            return Outcome::Failed;
        }
        self.pending
            .remove(name)
            .map_or(Outcome::Unknown, |declared| self.build(declared))
    }

    fn build(&mut self, declared: Declared<F, T>) -> Outcome<F> {
        let Declared {
            position,
            declaration,
        } = declared;
        self.path.push(OnPath {
            position,
            name: declaration.name.clone(),
        });
        let prerequisites = self.prerequisites_of(&declaration);
        self.path.pop();
        let dispatched = self.dispatched.held_by(&declaration);
        let call = call::call_for(&declaration, &self.leaf, dispatched);
        match (prerequisites, call) {
            (Some(prerequisites), Ok(call)) => {
                let step = step_of(declaration, call, dispatch_of(dispatched), prerequisites);
                self.finish(position, Rc::new(step))
            }
            (_, call) => {
                self.errors.extend(call.err());
                self.failed.insert(declaration.name);
                Outcome::Failed
            }
        }
    }

    /// The steps of every prerequisite, or `None` when one of them is unknown
    /// or failed; unknown ones are reported here, each named once, with what
    /// could be wired.
    fn prerequisites_of(
        &mut self,
        dependency: &DependencyDeclaration<F, T>,
    ) -> Option<Vec<Rc<Step<F>>>> {
        let outcomes: Vec<_> = dependency
            .prerequisites
            .iter()
            .map(|prerequisite| self.visit(&prerequisite.field))
            .collect();
        let missing = repeats(
            dependency
                .prerequisites
                .iter()
                .zip(&outcomes)
                .filter(is_unknown)
                .map(name_of_outcome),
        )
        .distinct;
        if !missing.is_empty() {
            self.errors.push(DomainError::UnknownPrerequisites {
                available: self.others_than(&dependency.name),
                dependency: dependency.name.clone(),
                missing,
            });
        }
        outcomes.into_iter().map(Outcome::into_step).collect()
    }

    fn others_than(&self, name: &DependencyName) -> Vec<DependencyName> {
        self.declared
            .iter()
            .filter(|declared| *declared != name)
            .cloned()
            .collect()
    }

    fn finish(&mut self, position: usize, step: Rc<Step<F>>) -> Outcome<F> {
        self.built.insert(step.name.clone(), Rc::clone(&step));
        self.construction_order.push(Placed {
            position,
            step: Rc::clone(&step),
        });
        Outcome::Built(step)
    }
}

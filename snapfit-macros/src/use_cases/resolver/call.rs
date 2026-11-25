//! Policy: which call builds a dependency's value.

use crate::domain::{
    Call, Construction, Constructor, DependencyDeclaration, DomainError, Implementation,
    TraitObjectProblem, TypeShape, Wrapping,
};
use crate::use_cases::leaf::is_leaf;

/// A constructor or a builder belongs to the type the field holds, to `T`
/// when the field holds `Arc<T>`, whose value is then wrapped, or to the
/// implementation the field names for its `Arc<dyn Trait>`. Any other value
/// that is a trait object needs a factory. A dispatched field is refused by
/// the feature gate, so its shape decides nothing yet.
pub(super) fn call_for<F>(
    dependency: &DependencyDeclaration<F>,
    leaf: &Constructor,
) -> Result<Call, DomainError> {
    Ok(match construction_of(dependency, leaf) {
        Construction::Factory if dependency.implementation.is_named() => {
            return Err(trait_object(
                dependency,
                TraitObjectProblem::ImplementationWithFactory,
            ));
        }
        Construction::Factory => Call::Factory,
        Construction::Builder(methods) => Call::Builder {
            methods,
            wrapping: wrapping_of(dependency)?,
        },
        Construction::Constructor(constructor) => Call::Constructor {
            constructor,
            wrapping: wrapping_of(dependency)?,
        },
    })
}

fn wrapping_of<F>(dependency: &DependencyDeclaration<F>) -> Result<Wrapping, DomainError> {
    if dependency.dispatch.is_some() {
        return Ok(Wrapping::None);
    }
    let problem = match (dependency.shape, dependency.implementation) {
        (TypeShape::Value | TypeShape::KeepsTraitObject, Implementation::OfField) => {
            return Ok(Wrapping::None);
        }
        (TypeShape::Arc | TypeShape::ArcKeepsTraitObject, Implementation::OfField)
        | (TypeShape::ArcTraitObject, Implementation::Named) => return Ok(Wrapping::Arc),
        (TypeShape::ArcTraitObject, Implementation::OfField) => {
            TraitObjectProblem::WithoutImplementation
        }
        (TypeShape::TraitObject, Implementation::OfField) => TraitObjectProblem::WithoutFactory,
        (TypeShape::TraitObject, Implementation::Named) => {
            TraitObjectProblem::ImplementationOutsideArc
        }
        (TypeShape::Value | TypeShape::Arc, Implementation::Named) => {
            TraitObjectProblem::ImplementationWithoutTraitObject
        }
        (TypeShape::KeepsTraitObject | TypeShape::ArcKeepsTraitObject, Implementation::Named) => {
            TraitObjectProblem::ImplementationKeptInside
        }
    };
    Err(trait_object(dependency, problem))
}

fn trait_object<F>(
    dependency: &DependencyDeclaration<F>,
    problem: TraitObjectProblem,
) -> DomainError {
    DomainError::TraitObject {
        dependency: dependency.name.clone(),
        problem,
    }
}

/// The field's own `build`; without one, the container's `leaf_build` for a
/// leaf, and the type's own `new` for any other field. The declaration rules
/// refuse a fallible leaf under another `leaf_build` than `new`, so no plan
/// is emitted for one.
fn construction_of<F>(dependency: &DependencyDeclaration<F>, leaf: &Constructor) -> Construction {
    match (&dependency.construction, is_leaf(dependency)) {
        (Some(construction), _) => construction.clone(),
        (None, true) => Construction::Constructor(leaf.clone()),
        (None, false) => Construction::Constructor(Constructor::New),
    }
}

#[cfg(test)]
mod tests;

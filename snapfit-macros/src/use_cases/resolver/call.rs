//! Policy: which call builds a dependency's value.

use crate::domain::{
    Call, Construction, Constructor, DependencyDeclaration, DomainError, Implementation,
    StaticDispatchProblem, TraitObjectProblem, TypeName, TypeShape, Wrapping,
};
use crate::use_cases::leaf::is_leaf;

/// A constructor or a builder belongs to the type the field holds, to `T`
/// when the field holds `Arc<T>`, whose value is then wrapped, or to the
/// implementation the field names for its `Arc<dyn Trait>` or for the type
/// `dispatched` it holds, whose variant then holds the value. Any other value
/// that is a trait object needs a factory. A dispatched field built by a
/// factory names the variants it may return.
pub(super) fn call_for<F, T>(
    dependency: &DependencyDeclaration<F, T>,
    leaf: &Constructor,
    dispatched: Option<&TypeName>,
) -> Result<Call, DomainError> {
    Ok(match construction_of(dependency, leaf) {
        Construction::Factory => factory_call(dependency, dispatched)?,
        Construction::Builder(methods) => Call::Builder {
            methods,
            wrapping: wrapping_of(dependency, dispatched)?,
        },
        Construction::Constructor(constructor) => Call::Constructor {
            constructor,
            wrapping: wrapping_of(dependency, dispatched)?,
        },
    })
}

/// A factory returns the field itself: the implementations a dispatched
/// field names are the variants it may return, and any other field names none.
fn factory_call<F, T>(
    dependency: &DependencyDeclaration<F, T>,
    dispatched: Option<&TypeName>,
) -> Result<Call, DomainError> {
    let problem = match (&dependency.implementation, dispatched) {
        (Implementation::OfField, _) | (_, Some(_)) => return Ok(Call::Factory),
        (Implementation::Named(_), None) => TraitObjectProblem::ImplementationWithFactory,
        (Implementation::Listed(_), None) => TraitObjectProblem::ImplementationList,
    };
    Err(trait_object(dependency, problem))
}

fn wrapping_of<F, T>(
    dependency: &DependencyDeclaration<F, T>,
    dispatched: Option<&TypeName>,
) -> Result<Wrapping, DomainError> {
    if let Some(name) = dispatched {
        return variant_wrapping(dependency, name);
    }
    let problem = match (dependency.shape, &dependency.implementation) {
        (_, Implementation::Listed(_)) => TraitObjectProblem::ImplementationList,
        (TypeShape::Value | TypeShape::KeepsTraitObject, Implementation::OfField) => {
            return Ok(Wrapping::None);
        }
        (TypeShape::Arc | TypeShape::ArcKeepsTraitObject, Implementation::OfField)
        | (TypeShape::ArcTraitObject, Implementation::Named(_)) => return Ok(Wrapping::Arc),
        (TypeShape::ArcTraitObject, Implementation::OfField) => {
            TraitObjectProblem::WithoutImplementation
        }
        (TypeShape::TraitObject, Implementation::OfField) => TraitObjectProblem::WithoutFactory,
        (TypeShape::TraitObject, Implementation::Named(_)) => {
            TraitObjectProblem::ImplementationOutsideArc
        }
        (TypeShape::Value | TypeShape::Arc, Implementation::Named(_)) => {
            TraitObjectProblem::ImplementationWithoutTraitObject
        }
        (
            TypeShape::KeepsTraitObject | TypeShape::ArcKeepsTraitObject,
            Implementation::Named(_),
        ) => TraitObjectProblem::ImplementationKeptInside,
    };
    Err(trait_object(dependency, problem))
}

/// A field typed `Arc<Name>` holds its variant in an `Arc`; one typed `Name`
/// holds it as it is, which is the only other shape a bare name has.
fn variant_wrapping<F, T>(
    dependency: &DependencyDeclaration<F, T>,
    name: &TypeName,
) -> Result<Wrapping, DomainError> {
    match (dependency.shape, &dependency.implementation) {
        (TypeShape::Arc, Implementation::Named(_)) => Ok(Wrapping::ArcOfVariant),
        (_, Implementation::Named(_)) => Ok(Wrapping::Variant),
        (_, Implementation::Listed(_)) => Err(StaticDispatchProblem::ListWithoutFactory {
            dependency: dependency.name.clone(),
            name: name.clone(),
        }
        .into()),
        (_, Implementation::OfField) => Err(StaticDispatchProblem::WithoutImplementation {
            dependency: dependency.name.clone(),
            name: name.clone(),
        }
        .into()),
    }
}

fn trait_object<F, T>(
    dependency: &DependencyDeclaration<F, T>,
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
fn construction_of<F, T>(
    dependency: &DependencyDeclaration<F, T>,
    leaf: &Constructor,
) -> Construction {
    match (&dependency.construction, is_leaf(dependency)) {
        (Some(construction), _) => construction.clone(),
        (None, true) => Construction::Constructor(leaf.clone()),
        (None, false) => Construction::Constructor(Constructor::New),
    }
}

#[cfg(test)]
mod tests;

//! Policy: which call builds a dependency's value.

use crate::domain::{
    Call, Construction, Constructor, DependencyDeclaration, DomainError, TypeShape, Wrapping,
};
use crate::use_cases::leaf::is_leaf;

/// A constructor or a builder belongs to the type the field holds, or to `T`
/// when the field holds `Arc<T>`, whose value is then wrapped. A trait object
/// has neither, so it needs a factory. A dispatched field is refused by the
/// feature gate, so its shape decides nothing yet.
pub(super) fn call_for<F>(
    dependency: &DependencyDeclaration<F>,
    leaf: &Constructor,
) -> Result<Call, DomainError> {
    Ok(match construction_of(dependency, leaf) {
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
    match (dependency.shape, dependency.dispatch.is_some()) {
        (_, true) | (TypeShape::Value, false) => Ok(Wrapping::None),
        (TypeShape::Arc, false) => Ok(Wrapping::Arc),
        (TypeShape::TraitObject, false) => Err(DomainError::TraitObjectWithoutFactory {
            dependency: dependency.name.clone(),
        }),
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

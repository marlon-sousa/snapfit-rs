//! Policy: which call builds a dependency's value.

use crate::domain::{
    Call, Construction, Constructor, DependencyDeclaration, DomainError, TypeShape, Wrapping,
};

/// A constructor belongs to the type the field holds, or to `T` when the field
/// holds `Arc<T>`, whose value is then wrapped. A trait object has no
/// constructor, so it needs a factory. A dispatched field and a builder are
/// refused by the feature gate, so their shape decides nothing yet.
pub(super) fn call_for<F>(
    dependency: &DependencyDeclaration<F>,
    leaf: &Constructor,
) -> Result<Call, DomainError> {
    let construction = construction_of(dependency, leaf);
    let constructor = match &construction {
        Construction::Factory => return Ok(Call::Factory),
        Construction::Constructor(constructor) => constructor.clone(),
        Construction::Builder => Constructor::New,
    };
    let refused = dependency.dispatch.is_some() || construction.is_builder();
    let wrapping = match (dependency.shape, refused) {
        (_, true) | (TypeShape::Value, false) => Wrapping::None,
        (TypeShape::Arc, false) => Wrapping::Arc,
        (TypeShape::TraitObject, false) => {
            return Err(DomainError::TraitObjectWithoutFactory {
                dependency: dependency.name.clone(),
            });
        }
    };
    Ok(Call::Constructor {
        constructor,
        wrapping,
    })
}

/// The field's own `build`; without one, the container's `leaf_build` for a
/// field with no `deps`, and the type's own `new` for any other.
fn construction_of<F>(dependency: &DependencyDeclaration<F>, leaf: &Constructor) -> Construction {
    let is_leaf = dependency.prerequisites.is_empty() && dependency.parent_prerequisites.is_empty();
    match (&dependency.construction, is_leaf) {
        (Some(construction), _) => construction.clone(),
        (None, true) => Construction::Constructor(leaf.clone()),
        (None, false) => Construction::Constructor(Constructor::New),
    }
}

#[cfg(test)]
mod tests;

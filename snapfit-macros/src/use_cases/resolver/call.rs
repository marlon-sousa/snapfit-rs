//! Policy: which call builds a dependency's value.

use crate::domain::{
    Call, Construction, DependencyDeclaration, DomainError, TraitObjectRemedy, TypeShape, Wrapping,
};

/// A constructor belongs to the type the field holds, or to `T` when the field
/// holds `Arc<T>`, whose value is then wrapped. A trait object has no
/// constructor, so it needs a builder, and is otherwise refused with `remedy`.
/// A dispatched field is refused by the feature gate, so its shape decides
/// nothing yet.
pub(super) fn call_for<F>(
    dependency: &DependencyDeclaration<F>,
    remedy: TraitObjectRemedy,
) -> Result<Call, DomainError> {
    let constructor = match &dependency.construction {
        Construction::Builder(function) => return Ok(Call::Builder(function.clone())),
        Construction::Constructor(constructor) => constructor.clone(),
    };
    let wrapping = match (dependency.shape, dependency.dispatch.is_some()) {
        (_, true) | (TypeShape::Value, false) => Wrapping::None,
        (TypeShape::Arc, false) => Wrapping::Arc,
        (TypeShape::TraitObject, false) => {
            let dependency = dependency.name.clone();
            return Err(match remedy {
                TraitObjectRemedy::ConcreteType => {
                    DomainError::TraitObjectNeedsConcreteType { dependency }
                }
                TraitObjectRemedy::Builder => DomainError::TraitObjectWithoutBuilder { dependency },
            });
        }
    };
    Ok(Call::Constructor {
        constructor,
        wrapping,
    })
}

#[cfg(test)]
mod tests;

//! Policy: which call builds a dependency's value.

use crate::domain::{Call, Construction, DependencyDeclaration, DomainError, TypeShape, Wrapping};
use crate::use_cases::FeatureGate;

/// A constructor belongs to the type the field holds, or to `T` when the field
/// holds `Arc<T>`, whose value is then wrapped. A trait object has no
/// constructor, so it needs a builder, and is otherwise refused
/// with the advice `gate` lets it follow.
/// A dispatched field is refused by the feature gate, so its shape decides
/// nothing yet.
pub(super) fn call_for<F>(
    dependency: &DependencyDeclaration<F>,
    gate: FeatureGate,
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
            return Err(match gate {
                FeatureGate::Stable => DomainError::TraitObjectNeedsConcreteType { dependency },
                FeatureGate::Unstable => DomainError::TraitObjectWithoutBuilder { dependency },
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

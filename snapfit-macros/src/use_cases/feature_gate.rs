//! Policy: which parts of a container declaration this build of snapfit accepts.

use crate::domain::{
    Construction, ContainerDeclaration, DependencyDeclaration, DependencyName, DomainError,
    Execution, Feature,
};

#[cfg(test)]
mod tests;

/// Planned features, with no code generation yet, are refused by every build.
/// Features whose API is not settled for 0.1.0 are accepted only by a build
/// with the `unstable` Cargo feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeatureGate {
    Stable,
    Unstable,
}

impl FeatureGate {
    /// Every use of a feature this build refuses, in declaration order.
    pub(crate) fn refusals<C, F>(self, container: &ContainerDeclaration<C, F>) -> Vec<DomainError> {
        let planned = planned_features(container).map(not_yet_implemented);
        let unstable = match self {
            Self::Stable => unstable_features(container),
            Self::Unstable => Vec::new(),
        };
        planned
            .chain(unstable.into_iter().map(requires_unstable))
            .collect()
    }
}

fn planned_features<C, F>(
    container: &ContainerDeclaration<C, F>,
) -> impl Iterator<Item = Feature> + '_ {
    let generic = container
        .type_parameters
        .is_declared()
        .then(|| Feature::GenericContainer {
            container: container.name.clone(),
        });
    let parent = container
        .parent
        .is_some()
        .then_some(Feature::ParentContainer);
    let introspection = container
        .introspection
        .is_on()
        .then_some(Feature::Introspection);
    generic
        .into_iter()
        .chain(parent)
        .chain(introspection)
        .chain(container.dependencies.iter().flat_map(planned_features_of))
}

fn planned_features_of<F>(dependency: &DependencyDeclaration<F>) -> Vec<Feature> {
    let dispatch = dependency.dispatch.is_some().then(|| Feature::Dispatch {
        dependency: dependency.name.clone(),
    });
    let parents = dependency
        .parent_prerequisites
        .iter()
        .map(|parent| parent_dependency(parent, dependency));
    dispatch.into_iter().chain(parents).collect()
}

fn parent_dependency<F>(parent: &DependencyName, dependency: &DependencyDeclaration<F>) -> Feature {
    Feature::ParentDependency {
        parent: parent.clone(),
        dependency: dependency.name.clone(),
    }
}

fn unstable_features<C, F>(container: &ContainerDeclaration<C, F>) -> Vec<Feature> {
    let async_mode = (container.mode == Some(Execution::Async)).then_some(Feature::AsyncMode);
    async_mode
        .into_iter()
        .chain(container.dependencies.iter().flat_map(unstable_features_of))
        .collect()
}

fn unstable_features_of<F>(dependency: &DependencyDeclaration<F>) -> Vec<Feature> {
    let name = &dependency.name;
    let is_async = dependency
        .execution
        .is_async()
        .then(|| Feature::AsyncDependency {
            dependency: name.clone(),
        });
    let is_transient = dependency
        .lifecycle
        .is_transient()
        .then(|| Feature::Transient {
            dependency: name.clone(),
        });
    let builder = match &dependency.construction {
        Construction::Builder(builder) => Some(Feature::Builder {
            builder: builder.clone(),
            dependency: name.clone(),
        }),
        Construction::Constructor(_) => None,
    };
    [is_async, is_transient, builder]
        .into_iter()
        .flatten()
        .collect()
}

fn not_yet_implemented(feature: Feature) -> DomainError {
    DomainError::NotYetImplemented { feature }
}

fn requires_unstable(feature: Feature) -> DomainError {
    DomainError::RequiresUnstable { feature }
}

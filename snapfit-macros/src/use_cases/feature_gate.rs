//! Policy: which parts of a container declaration this build of snapfit accepts.

use crate::domain::{
    Construction, ContainerDeclaration, DependencyDeclaration, DependencyName, DomainError,
    Execution, Feature, FeatureGate,
};

#[cfg(test)]
mod tests;

impl FeatureGate {
    /// Every use of a feature this build refuses, in declaration order.
    /// Planned features, with no code generation yet, are refused by every
    /// build. Features whose API is not settled for 0.1.0 are accepted only by
    /// a build with the `unstable` Cargo feature.
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
        .then(|| generic_container(container));
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
    let name = dependency.name.clone();
    let dispatch = dependency.dispatch.is_some().then_some(Feature::Dispatch {
        dependency: name.clone(),
    });
    let async_build = match (&dependency.construction, dependency.execution) {
        (Some(Construction::Factory), Execution::Async) => Some(Feature::AsyncFactory {
            dependency: name.clone(),
        }),
        (Some(Construction::Builder(_)), Execution::Async) => Some(Feature::AsyncBuilder {
            dependency: name.clone(),
        }),
        _ => None,
    };
    let fallible_transient = (dependency.lifecycle.is_transient()
        && dependency.fallibility.is_fallible())
    .then_some(Feature::FallibleTransient { dependency: name });
    let parents = dependency
        .parent_prerequisites
        .iter()
        .map(|parent| parent_dependency(parent, dependency));
    dispatch
        .into_iter()
        .chain(async_build)
        .chain(fallible_transient)
        .chain(parents)
        .collect()
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
        .then(|| async_dependency(name));
    let is_transient = dependency.lifecycle.is_transient().then(|| transient(name));
    [is_async, is_transient].into_iter().flatten().collect()
}

fn generic_container<C, F>(container: &ContainerDeclaration<C, F>) -> Feature {
    Feature::GenericContainer {
        container: container.name.clone(),
    }
}

fn async_dependency(dependency: &DependencyName) -> Feature {
    Feature::AsyncDependency {
        dependency: dependency.clone(),
    }
}

fn transient(dependency: &DependencyName) -> Feature {
    Feature::Transient {
        dependency: dependency.clone(),
    }
}

fn not_yet_implemented(feature: Feature) -> DomainError {
    DomainError::NotYetImplemented { feature }
}

fn requires_unstable(feature: Feature) -> DomainError {
    DomainError::RequiresUnstable { feature }
}

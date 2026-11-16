use crate::domain::{ContainerIR, DependencyIR, DependencyName, DomainError};

type Finding = (String, Option<DependencyName>);

#[cfg(test)]
mod tests;

/// Decides which parts of a container declaration this build of snapfit accepts.
///
/// Planned features with no code generation yet are always rejected. Features
/// whose API is not settled for 0.1.0 are accepted only when the `unstable`
/// Cargo feature is enabled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FeatureGate {
    unstable_enabled: bool,
}

impl FeatureGate {
    pub(crate) const fn new(unstable_enabled: bool) -> Self {
        Self { unstable_enabled }
    }

    pub(crate) fn check(&self, container: &ContainerIR) -> Result<(), DomainError> {
        Self::first_planned_feature(container).map_or(Ok(()), |(feature, dependency)| {
            Err(DomainError::NotYetImplemented {
                feature,
                dependency,
            })
        })?;
        if self.unstable_enabled {
            return Ok(());
        }
        Self::first_unstable_feature(container).map_or(Ok(()), |(feature, dependency)| {
            Err(DomainError::RequiresUnstable {
                feature,
                dependency,
            })
        })
    }

    fn first_planned_feature(container: &ContainerIR) -> Option<Finding> {
        if container.parent_type.is_some() {
            return Some(("#[container(parent = ...)]".to_string(), None));
        }
        if container.introspection {
            return Some(("#[container(introspection = true)]".to_string(), None));
        }
        container
            .dependencies
            .iter()
            .find_map(Self::planned_feature_of)
    }

    fn planned_feature_of(dep: &DependencyIR) -> Option<Finding> {
        if dep.trait_dispatch.is_enum() {
            let feature = format!("#[dispatch(...)] on dependency '{}'", dep.name);
            return Some((feature, Some(dep.name.clone())));
        }
        dep.prerequisites
            .iter()
            .find(|prereq| prereq.as_str().starts_with("parent("))
            .map(|prereq| {
                let feature = format!("parent dependency '{prereq}' on dependency '{}'", dep.name);
                (feature, Some(dep.name.clone()))
            })
    }

    fn first_unstable_feature(container: &ContainerIR) -> Option<Finding> {
        container
            .dependencies
            .iter()
            .find_map(|dep| Self::unstable_feature_of(dep).map(|f| (f, Some(dep.name.clone()))))
            .or_else(|| {
                container
                    .is_async
                    .then(|| ("#[container(mode = \"async\")]".to_string(), None))
            })
    }

    fn unstable_feature_of(dep: &DependencyIR) -> Option<String> {
        if dep.is_async {
            return Some(format!("async dependency '{}'", dep.name));
        }
        if dep.lifecycle.is_transient() {
            return Some(format!("transient dependency '{}'", dep.name));
        }
        dep.custom_builder
            .as_ref()
            .map(|builder| format!("custom builder '{builder}' on dependency '{}'", dep.name))
    }
}

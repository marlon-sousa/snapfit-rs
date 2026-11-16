use crate::domain::{ContainerIR, DependencyIR, DomainError};

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
        Self::first_planned_feature(container).map_or(Ok(()), |feature| {
            Err(DomainError::NotYetImplemented(feature))
        })?;
        if self.unstable_enabled {
            return Ok(());
        }
        Self::first_unstable_feature(container).map_or(Ok(()), |feature| {
            Err(DomainError::RequiresUnstable(feature))
        })
    }

    fn first_planned_feature(container: &ContainerIR) -> Option<String> {
        if container.parent_type.is_some() {
            return Some("#[container(parent = ...)]".to_string());
        }
        if container.introspection {
            return Some("#[container(introspection = true)]".to_string());
        }
        container
            .dependencies
            .iter()
            .find_map(Self::planned_feature_of)
    }

    fn planned_feature_of(dep: &DependencyIR) -> Option<String> {
        if dep.trait_dispatch.is_enum() {
            return Some(format!("#[dispatch(...)] on dependency '{}'", dep.name));
        }
        dep.prerequisites
            .iter()
            .find(|prereq| prereq.as_str().starts_with("parent("))
            .map(|prereq| format!("parent dependency '{prereq}' on dependency '{}'", dep.name))
    }

    fn first_unstable_feature(container: &ContainerIR) -> Option<String> {
        container
            .dependencies
            .iter()
            .find_map(Self::unstable_feature_of)
            .or_else(|| {
                container
                    .is_async
                    .then(|| "#[container(mode = \"async\")]".to_string())
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

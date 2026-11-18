use super::*;

#[test]
fn a_declared_dependency_starts_as_a_synchronous_singleton_with_no_build_of_its_own() {
    let dependency = DependencyDeclaration::new(DependencyName::new("db").unwrap(), ());

    assert_eq!(dependency.lifecycle, Lifecycle::Singleton);
    assert_eq!(dependency.execution, Execution::Sync);
    assert_eq!(dependency.construction, None);
    assert_eq!(dependency.shape, TypeShape::Value);
    assert!(dependency.prerequisites.is_empty());
    assert!(dependency.dispatch.is_none());
}

#[test]
fn a_declared_container_leaves_its_mode_to_its_dependencies() {
    let container: ContainerDeclaration<(), ()> =
        ContainerDeclaration::new(ContainerName::new("App").unwrap(), ());

    assert_eq!(container.mode, None);
    assert_eq!(container.parent, None);
    assert!(container.introspection.is_off());
    assert!(container.dependencies.is_empty());
}

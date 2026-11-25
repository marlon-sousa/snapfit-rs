use super::*;
use crate::test_support::labelled;

#[test]
fn a_declared_dependency_starts_as_a_synchronous_singleton_with_no_build_of_its_own() {
    let dependency: DependencyDeclaration<()> =
        DependencyDeclaration::new(DependencyName::new("db").unwrap(), ());

    assert_eq!(dependency.lifecycle, Lifecycle::Singleton);
    assert_eq!(dependency.execution, Execution::Sync);
    assert_eq!(dependency.construction, None);
    assert_eq!(dependency.shape, TypeShape::Value);
    assert!(dependency.prerequisites.is_empty());
    assert!(dependency.written_type.is_other());
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

#[test]
fn a_prerequisite_is_passed_to_the_setter_its_label_names_or_to_its_own() {
    let field = DependencyName::new("folder_files").unwrap();
    let bare = Prerequisite::new(field);

    assert_eq!(bare.setter().as_str(), "folder_files");
    assert_eq!(labelled("folder_files", "disk").setter().as_str(), "disk");
}

#[test]
fn new_is_the_type_s_own_new_whether_left_out_or_written_as_a_constructor() {
    assert!(Constructor::New.is_new());
    assert!(Constructor::Named(FunctionName::new("new").unwrap()).is_new());
    assert!(!Constructor::Named(FunctionName::new("open").unwrap()).is_new());
    assert!(!Constructor::Default.is_new());
}

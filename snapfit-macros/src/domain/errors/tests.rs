use super::*;

fn dep_name(s: &str) -> DependencyName {
    DependencyName::new(s).unwrap()
}

#[test]
fn test_domain_error_display() {
    let cycle_err = DomainError::CycleDetected(vec![dep_name("a"), dep_name("b"), dep_name("a")]);
    assert_eq!(
        cycle_err.to_string(),
        "dependency cycle detected: a -> b -> a"
    );

    let missing_err = DomainError::MissingPrerequisites {
        dependency: dep_name("db"),
        missing: vec![dep_name("confog")],
        available: vec![dep_name("config"), dep_name("user_service")],
    };
    assert_eq!(
        missing_err.to_string(),
        "unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config, user_service]"
    );

    let dup_err = DomainError::DuplicateDependency(dep_name("cache"));
    assert_eq!(dup_err.to_string(), "duplicate dependency name: 'cache'");

    let invalid_err = DomainError::InvalidConfiguration("mode cannot be empty".into());
    assert_eq!(
        invalid_err.to_string(),
        "invalid configuration: mode cannot be empty"
    );

    let unstable_err = DomainError::RequiresUnstable("transient dependency 'cart'".into());
    assert_eq!(
        unstable_err.to_string(),
        "transient dependency 'cart' is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    );

    let planned_err = DomainError::NotYetImplemented("#[container(parent = ...)]".into());
    assert_eq!(
        planned_err.to_string(),
        "#[container(parent = ...)] is planned for a future version of snapfit and is not implemented yet"
    );
}

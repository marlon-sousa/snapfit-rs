use super::*;

#[test]
fn test_lifecycle_variants_and_predicates() {
    let singleton = Lifecycle::Singleton;
    assert!(singleton.is_singleton());
    assert!(!singleton.is_transient());
    assert_eq!(singleton.as_str(), "singleton");

    let transient = Lifecycle::Transient;
    assert!(!transient.is_singleton());
    assert!(transient.is_transient());
    assert_eq!(transient.as_str(), "transient");

    assert_eq!(Lifecycle::default(), Lifecycle::Singleton);
}

#[test]
fn test_trait_dispatch_variants() {
    let none = TraitDispatch::None;
    assert_eq!(none, TraitDispatch::default());

    let enum_dispatch = TraitDispatch::Enum {
        trait_name: "Repository".to_string(),
        impls: vec!["PostgresRepo".to_string(), "MockRepo".to_string()],
    };
    if let TraitDispatch::Enum { trait_name, impls } = &enum_dispatch {
        assert_eq!(trait_name, "Repository");
        assert_eq!(impls.len(), 2);
    } else {
        panic!("expected TraitDispatch::Enum");
    }

    let dyn_dispatch = TraitDispatch::Dyn {
        trait_type: "dyn Service".to_string(),
    };
    if let TraitDispatch::Dyn { trait_type } = &dyn_dispatch {
        assert_eq!(trait_type, "dyn Service");
    } else {
        panic!("expected TraitDispatch::Dyn");
    }
}

#[test]
fn test_dependency_ir_creation() {
    let mut dep = DependencyIR::new("database", "Arc<Database>");
    assert_eq!(*dep.name, "database");
    assert_eq!(*dep.type_string, "Arc<Database>");
    assert!(dep.prerequisites.is_empty());
    assert_eq!(dep.lifecycle, Lifecycle::Singleton);
    assert!(!dep.is_async);
    assert_eq!(dep.constructor_name, "new");
    assert_eq!(dep.custom_builder, None);
    assert_eq!(dep.trait_dispatch, TraitDispatch::None);

    dep.prerequisites.push("config".into());
    dep.lifecycle = Lifecycle::Transient;
    dep.is_async = true;
    dep.constructor_name = "connect".to_string();

    assert_eq!(dep.prerequisites, vec!["config".into()]);
    assert_eq!(dep.lifecycle, Lifecycle::Transient);
    assert!(dep.is_async);
    assert_eq!(dep.constructor_name, "connect");
}

#[test]
fn test_container_ir_creation() {
    let mut container = ContainerIR::new("AppContainer");
    assert_eq!(*container.name, "AppContainer");
    assert!(!container.is_async);
    assert_eq!(container.parent_type, None);
    assert!(!container.introspection);
    assert!(container.dependencies.is_empty());

    let config_dep = DependencyIR::new("config", "Arc<Config>");
    let db_dep = DependencyIR::new("database", "Arc<Database>");
    container.dependencies.push(config_dep);
    container.dependencies.push(db_dep);
    container.is_async = true;
    container.introspection = true;

    assert_eq!(container.dependencies.len(), 2);
    assert!(container.is_async);
    assert!(container.introspection);
}

#[test]
fn test_construction_plan_and_steps() {
    let mut plan = ConstructionPlan::new("AppContainer", true, "AppContainerKey");
    assert_eq!(*plan.container_name, "AppContainer");
    assert!(plan.is_async);
    assert_eq!(plan.key_enum_name, "AppContainerKey");
    assert!(plan.ordered_steps.is_empty());
    assert!(plan.parallel_batches.is_empty());

    let step = InstantiationStep::new(
        "config",
        "Arc<Config>",
        "Config::new()",
        false,
        Lifecycle::Singleton,
    );
    plan.ordered_steps.push(step);
    plan.parallel_batches.push(vec!["config".into()]);

    assert_eq!(plan.ordered_steps.len(), 1);
    assert_eq!(*plan.ordered_steps[0].dep_name, "config");
    assert_eq!(plan.ordered_steps[0].lifecycle, Lifecycle::Singleton);
    assert_eq!(plan.parallel_batches.len(), 1);
}

#[test]
fn test_domain_error_display() {
    let cycle_err = DomainError::CycleDetected(vec!["a".into(), "b".into(), "a".into()]);
    assert_eq!(
        cycle_err.to_string(),
        "dependency cycle detected: a -> b -> a"
    );

    let missing_err = DomainError::MissingPrerequisites {
        dependency: "db".into(),
        missing: vec!["config".into(), "pool".into()],
    };
    assert_eq!(
        missing_err.to_string(),
        "dependency 'db' is missing prerequisites: config, pool"
    );

    let dup_err = DomainError::DuplicateDependency("cache".into());
    assert_eq!(dup_err.to_string(), "duplicate dependency name: 'cache'");

    let invalid_err = DomainError::InvalidConfiguration("mode cannot be empty".into());
    assert_eq!(
        invalid_err.to_string(),
        "invalid configuration: mode cannot be empty"
    );
}

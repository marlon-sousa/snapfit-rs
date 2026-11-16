use super::*;

fn dep_name(s: &str) -> DependencyName {
    DependencyName::new(s).unwrap()
}

fn type_name(s: &str) -> TypeName {
    TypeName::new(s).unwrap()
}

fn container_name(s: &str) -> ContainerName {
    ContainerName::new(s).unwrap()
}

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
    assert!(enum_dispatch.is_enum());

    let dyn_dispatch = TraitDispatch::Dyn {
        trait_type: "dyn Service".to_string(),
    };
    assert!(dyn_dispatch.is_dyn());
}

#[test]
fn test_semantic_type_non_blank_invariants() {
    assert!(DependencyName::new("").is_err());
    assert!(DependencyName::new("   ").is_err());
    assert!(DependencyName::new("\t\n").is_err());
    assert!(DependencyName::new("valid_dep").is_ok());

    assert!(TypeName::new("").is_err());
    assert!(TypeName::new("   ").is_err());
    assert!(TypeName::new("Arc<Database>").is_ok());

    assert!(ContainerName::new("").is_err());
    assert!(ContainerName::new("   ").is_err());
    assert!(ContainerName::new("AppContainer").is_ok());
}

#[test]
fn test_dependency_and_container_ir_creation() {
    let mut dep = DependencyIR::new(dep_name("database"), type_name("Arc<Database>"));
    assert_eq!(*dep.name, "database");
    assert_eq!(*dep.type_string, "Arc<Database>");
    assert_eq!(dep.lifecycle, Lifecycle::Singleton);
    assert_eq!(dep.constructor_name, "new");

    dep.prerequisites.push(dep_name("config"));
    dep.lifecycle = Lifecycle::Transient;
    dep.is_async = true;

    let mut container = ContainerIR::new(container_name("AppContainer"));
    container.dependencies.push(dep);
    container.is_async = true;
    container.introspection = true;

    assert_eq!(container.dependencies.len(), 1);
    assert!(container.is_async);
    assert!(container.introspection);
}

use super::*;
use crate::domain::{ContainerName, DependencyName, Lifecycle, TraitDispatch, TypeName};

fn container_with(deps: Vec<DependencyIR>) -> ContainerIR {
    let mut container = ContainerIR::new(ContainerName::new("App").unwrap());
    container.dependencies = deps;
    container
}

fn dep(name: &str) -> DependencyIR {
    DependencyIR::new(
        DependencyName::new(name).unwrap(),
        TypeName::new("Arc<Service>").unwrap(),
    )
}

fn stable() -> FeatureGate {
    FeatureGate::new(false)
}

fn unstable() -> FeatureGate {
    FeatureGate::new(true)
}

fn requires_unstable(gate: FeatureGate, container: &ContainerIR) -> String {
    match gate.check(container) {
        Err(DomainError::RequiresUnstable { feature, .. }) => feature,
        other => panic!("expected RequiresUnstable, got {other:?}"),
    }
}

fn not_yet_implemented(gate: FeatureGate, container: &ContainerIR) -> String {
    match gate.check(container) {
        Err(DomainError::NotYetImplemented { feature, .. }) => feature,
        other => panic!("expected NotYetImplemented, got {other:?}"),
    }
}

#[test]
fn test_stable_singletons_pass_both_gates() {
    let mut db = dep("db");
    db.prerequisites = vec![DependencyName::new("config").unwrap()];
    db.constructor_name = "connect".to_string();
    let container = container_with(vec![dep("config"), db]);

    assert_eq!(stable().check(&container), Ok(()));
    assert_eq!(unstable().check(&container), Ok(()));
}

#[test]
fn test_async_transient_and_builder_require_unstable() {
    let mut async_dep = dep("db");
    async_dep.is_async = true;
    let mut transient = dep("cart");
    transient.lifecycle = Lifecycle::Transient;
    let mut built = dep("auth");
    built.custom_builder = Some("build_auth".to_string());

    let mut async_mode = container_with(vec![dep("config")]);
    async_mode.is_async = true;

    let cases = [
        (container_with(vec![async_dep]), "async dependency 'db'"),
        (
            container_with(vec![transient]),
            "transient dependency 'cart'",
        ),
        (
            container_with(vec![built]),
            "custom builder 'build_auth' on dependency 'auth'",
        ),
        (async_mode, "#[container(mode = \"async\")]"),
    ];

    cases.into_iter().for_each(|(container, expected)| {
        assert_eq!(requires_unstable(stable(), &container), expected);
        assert_eq!(unstable().check(&container), Ok(()));
    });
}

#[test]
fn test_planned_features_are_rejected_even_with_unstable() {
    let mut with_parent = container_with(vec![dep("config")]);
    with_parent.parent_type = Some(TypeName::new("RootContainer").unwrap());

    let mut with_introspection = container_with(vec![dep("config")]);
    with_introspection.introspection = true;

    let mut dispatched = dep("store");
    dispatched.trait_dispatch = TraitDispatch::Enum {
        trait_name: "Store".to_string(),
        impls: vec!["Postgres".to_string()],
    };

    let mut parent_prereq = dep("audit");
    parent_prereq.prerequisites = vec![DependencyName::new("parent(db)").unwrap()];

    let cases = [
        (with_parent, "#[container(parent = ...)]"),
        (with_introspection, "#[container(introspection = true)]"),
        (
            container_with(vec![dispatched]),
            "#[dispatch(...)] on dependency 'store'",
        ),
        (
            container_with(vec![parent_prereq]),
            "parent dependency 'parent(db)' on dependency 'audit'",
        ),
    ];

    cases.into_iter().for_each(|(container, expected)| {
        assert_eq!(not_yet_implemented(stable(), &container), expected);
        assert_eq!(not_yet_implemented(unstable(), &container), expected);
    });
}

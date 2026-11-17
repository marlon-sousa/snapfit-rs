use super::*;
use crate::domain::{Constructor, EnumDispatch, FunctionName, Introspection, Lifecycle, TypeName};
use crate::test_support::{container, dependency};

fn features(errors: &[DomainError]) -> Vec<String> {
    errors.iter().map(ToString::to_string).collect()
}

fn unstable(feature: &str) -> String {
    format!(
        "{feature} is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    )
}

fn planned(feature: &str) -> String {
    format!("{feature} is planned for a future version of snapfit and is not implemented yet")
}

#[test]
fn stable_singletons_pass_both_gates() {
    let mut db = dependency("db", &[]);
    db.prerequisites = vec![DependencyName::new("config").unwrap()];
    db.construction =
        Construction::Constructor(Constructor::Named(FunctionName::new("connect").unwrap()));
    let container = container(vec![dependency("config", &[]), db]);

    assert!(FeatureGate::Stable.refusals(&container).is_empty());
    assert!(FeatureGate::Unstable.refusals(&container).is_empty());
}

#[test]
fn a_stable_build_reports_every_unstable_feature_in_declaration_order() {
    let mut db = dependency("db", &[]);
    db.execution = Execution::Async;
    let mut cart = dependency("cart", &[]);
    cart.lifecycle = Lifecycle::Transient;
    cart.construction = Construction::Builder(FunctionName::new("build_cart").unwrap());
    let mut async_mode = container(vec![db, cart]);
    async_mode.mode = Some(Execution::Async);

    assert_eq!(
        features(&FeatureGate::Stable.refusals(&async_mode)),
        [
            unstable("#[container(mode = \"async\")]"),
            unstable("async dependency 'db'"),
            unstable("transient dependency 'cart'"),
            unstable("custom builder 'build_cart' on dependency 'cart'"),
        ]
    );
    assert!(FeatureGate::Unstable.refusals(&async_mode).is_empty());
}

#[test]
fn every_build_reports_every_planned_feature() {
    let mut store = dependency("store", &[]);
    let store_trait = TypeName::new("Store").unwrap();
    store.dispatch = Some(EnumDispatch::new(store_trait.clone(), vec![store_trait]));
    let mut audit = dependency("audit", &[]);
    audit.parent_prerequisites = vec![DependencyName::new("db").unwrap()];
    let mut planned_container = container(vec![store, audit]);
    planned_container.parent = Some(TypeName::new("Root").unwrap());
    planned_container.introspection = Introspection::On;

    let expected = [
        planned("#[container(parent = ...)]"),
        planned("#[container(introspection = true)]"),
        planned("#[dispatch(...)] on dependency 'store'"),
        planned("parent dependency 'parent(db)' on dependency 'audit'"),
    ];
    assert_eq!(
        features(&FeatureGate::Stable.refusals(&planned_container)),
        expected
    );
    assert_eq!(
        features(&FeatureGate::Unstable.refusals(&planned_container)),
        expected
    );
}

#[test]
fn a_refusal_points_at_the_field_that_uses_the_feature() {
    let mut cart = dependency("cart", &[]);
    cart.lifecycle = Lifecycle::Transient;

    let refusals = FeatureGate::Stable.refusals(&container(vec![cart]));

    assert_eq!(
        refusals[0].subject(),
        Some(&DependencyName::new("cart").unwrap())
    );
}

#[test]
fn a_generic_container_is_planned_and_named() {
    let mut generic = container(vec![dependency("store", &[])]);
    generic.type_parameters = crate::domain::TypeParameters::Declared;

    assert_eq!(
        features(&FeatureGate::Unstable.refusals(&generic)),
        [planned("generic parameters on container 'App'")]
    );
}

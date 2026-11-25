use super::*;
use crate::domain::{AsyncProblem, DomainError, Execution, Lifecycle, TypeShape};
use crate::test_support::{container, dependency};

#[test]
fn a_valid_container_is_planned_in_dependency_order() {
    let plan = plan_container(
        container(vec![
            dependency("service", &["db"]),
            dependency("db", &["config"]),
            dependency("config", &[]),
        ]),
        FeatureGate::Stable,
    )
    .unwrap();

    let order: Vec<_> = plan
        .construction_order
        .iter()
        .map(|step| step.name.as_str())
        .collect();
    assert_eq!(order, ["config", "db", "service"]);
    assert_eq!(plan.execution, Execution::Sync);
}

#[test]
fn an_async_dependency_makes_the_plan_async() {
    let mut db = dependency("db", &[]);
    db.execution = Execution::Async;

    let plan = plan_container(container(vec![db]), FeatureGate::Unstable).unwrap();

    assert_eq!(plan.execution, Execution::Async);
}

#[test]
fn refusals_mode_errors_and_graph_errors_are_reported_together() {
    let mut cart = dependency("cart", &[]);
    cart.lifecycle = Lifecycle::Transient;
    let mut db = dependency("db", &["confog"]);
    db.execution = Execution::Async;
    let mut declared = container(vec![cart, db, dependency("loop", &["loop"])]);
    declared.mode = Some(Execution::Sync);

    let errors: Vec<_> = plan_container(declared, FeatureGate::Stable)
        .unwrap_err()
        .into_iter()
        .collect();

    assert!(matches!(errors[0], DomainError::RequiresUnstable { .. }));
    assert!(matches!(errors[1], DomainError::RequiresUnstable { .. }));
    assert!(matches!(
        errors[2],
        DomainError::Async(AsyncProblem::InSyncContainer(_))
    ));
    assert!(matches!(
        errors[3],
        DomainError::UnknownPrerequisites { .. }
    ));
    assert!(matches!(errors[4], DomainError::DependencyCycle { .. }));
    assert_eq!(errors.len(), 5);
}

#[test]
fn both_builds_refuse_a_trait_object_without_a_factory_with_the_same_advice() {
    let mut store = dependency("store", &[]);
    store.shape = TypeShape::TraitObject;
    let refusal = |gate| {
        plan_container(container(vec![store.clone()]), gate)
            .unwrap_err()
            .into_iter()
            .collect::<Vec<_>>()
    };

    for gate in [FeatureGate::Stable, FeatureGate::Unstable] {
        assert!(
            matches!(refusal(gate)[..], [DomainError::TraitObject { .. }]),
            "{gate:?}"
        );
    }
}

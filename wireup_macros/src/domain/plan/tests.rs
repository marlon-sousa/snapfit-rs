use super::*;

#[test]
fn test_construction_plan_and_steps() {
    let container = ContainerName::new("AppContainer").unwrap();
    let dep = DependencyName::new("config").unwrap();
    let ty = TypeName::new("Arc<Config>").unwrap();

    let mut plan = ConstructionPlan::new(container, true, "AppContainerKey");
    assert_eq!(*plan.container_name, "AppContainer");
    assert!(plan.is_async);
    assert_eq!(plan.key_enum_name, "AppContainerKey");
    assert!(plan.ordered_steps.is_empty());

    let step = InstantiationStep::new(
        dep.clone(),
        ty,
        "Config::new()",
        false,
        Lifecycle::Singleton,
    );
    plan.ordered_steps.push(step);
    plan.parallel_batches.push(vec![dep]);

    assert_eq!(plan.ordered_steps.len(), 1);
    assert_eq!(*plan.ordered_steps[0].dep_name, "config");
    assert_eq!(plan.ordered_steps[0].lifecycle, Lifecycle::Singleton);
    assert_eq!(plan.parallel_batches.len(), 1);
}

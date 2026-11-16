use super::*;

#[test]
fn test_construction_plan_and_steps() {
    let container = ContainerName::new("AppContainer").unwrap();
    let config = DependencyName::new("config").unwrap();
    let db = DependencyName::new("db").unwrap();

    let mut plan = ConstructionPlan::new(container, true);
    assert_eq!(*plan.container_name, "AppContainer");
    assert!(plan.is_async);
    assert!(plan.ordered_steps.is_empty());

    plan.ordered_steps.push(InstantiationStep::new(
        config.clone(),
        TypeName::new("Arc<Config>").unwrap(),
        Construction::Constructor("new".to_string()),
        Vec::new(),
        false,
        Lifecycle::Singleton,
    ));
    plan.ordered_steps.push(InstantiationStep::new(
        db.clone(),
        TypeName::new("Arc<Database>").unwrap(),
        Construction::Builder("build_db".to_string()),
        vec![config.clone()],
        false,
        Lifecycle::Singleton,
    ));

    assert_eq!(plan.step(&db).unwrap().prerequisites, vec![config.clone()]);
    assert_eq!(
        plan.step(&config).unwrap().construction,
        Construction::Constructor("new".to_string())
    );
    assert!(
        plan.step(&DependencyName::new("missing").unwrap())
            .is_none()
    );
}

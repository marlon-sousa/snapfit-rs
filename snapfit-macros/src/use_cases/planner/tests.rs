use super::*;
use crate::domain::{
    ContainerIR, ContainerName, DependencyIR, DependencyName, Lifecycle, TypeName,
};

fn dep_name(s: &str) -> DependencyName {
    DependencyName::new(s).unwrap()
}

fn make_dep(name: &str, ty: &str, prereqs: &[&str]) -> DependencyIR {
    let mut dep = DependencyIR::new(dep_name(name), TypeName::new(ty).unwrap());
    dep.prerequisites = prereqs.iter().map(|p| dep_name(p)).collect();
    dep
}

#[test]
fn test_execution_planner_builds_typed_steps_and_infers_async() {
    let container = ContainerIR::new(ContainerName::new("AppContainer").unwrap());
    let config = make_dep("config", "std::sync::Arc<Config<Prod>>", &[]);

    let mut db = make_dep("db", "Arc<Database>", &["config", "config"]);
    db.constructor_name = "connect".to_string();
    db.is_async = true;

    let mut auth = make_dep("auth", "Arc<AuthService>", &["config", "db"]);
    auth.custom_builder = Some("build_auth".to_string());

    let mut controller = make_dep("controller", "UserController", &["auth"]);
    controller.lifecycle = Lifecycle::Transient;

    let plan = ExecutionPlanner::new()
        .build_plan(&container, vec![config, db, auth, controller])
        .unwrap();

    assert!(plan.is_async);
    let steps = &plan.ordered_steps;
    assert_eq!(steps.len(), 4);
    assert_eq!(
        steps[0].construction,
        Construction::Constructor("new".into())
    );
    assert!(steps[0].prerequisites.is_empty());
    assert_eq!(
        steps[1].construction,
        Construction::Constructor("connect".into())
    );
    assert_eq!(
        steps[1].prerequisites,
        vec![dep_name("config"), dep_name("config")]
    );
    assert!(steps[1].is_async);
    assert_eq!(
        steps[2].construction,
        Construction::Builder("build_auth".into())
    );
    assert_eq!(
        steps[2].prerequisites,
        vec![dep_name("config"), dep_name("db")]
    );
    assert_eq!(steps[3].lifecycle, Lifecycle::Transient);
}

#[test]
fn test_sync_container_without_async_dependencies_stays_sync() {
    let container = ContainerIR::new(ContainerName::new("App").unwrap());
    let plan = ExecutionPlanner::new()
        .build_plan(&container, vec![make_dep("config", "Config", &[])])
        .unwrap();
    assert!(!plan.is_async);
}

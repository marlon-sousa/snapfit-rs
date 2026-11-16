use super::*;
use crate::domain::{
    ContainerIR, ContainerName, DependencyIR, DependencyName, Lifecycle, TypeName,
};

fn dep_name(s: &str) -> DependencyName {
    DependencyName::new(s).unwrap()
}

fn type_name(s: &str) -> TypeName {
    TypeName::new(s).unwrap()
}

fn container_name(s: &str) -> ContainerName {
    ContainerName::new(s).unwrap()
}

fn make_dep(name: &str, ty: &str, prereqs: &[&str]) -> DependencyIR {
    let mut dep = DependencyIR::new(dep_name(name), type_name(ty));
    dep.prerequisites = prereqs.iter().map(|p| dep_name(p)).collect();
    dep
}

#[test]
fn test_strip_arc_wrapper_variants() {
    assert_eq!(ExecutionPlanner::strip_arc_wrapper("Arc<Config>"), "Config");
    assert_eq!(
        ExecutionPlanner::strip_arc_wrapper("std::sync::Arc<  Config<Prod>  >"),
        "Config<Prod>"
    );
    assert_eq!(
        ExecutionPlanner::strip_arc_wrapper("alloc::sync::Arc<Database>"),
        "Database"
    );
    assert_eq!(
        ExecutionPlanner::strip_arc_wrapper("UserController"),
        "UserController"
    );
}

#[test]
fn test_execution_planner_builds_steps_and_infers_async() {
    let container = ContainerIR::new(container_name("AppContainer"));
    let config = make_dep("config", "std::sync::Arc<  Config<Prod>  >", &[]);

    let mut db = make_dep("db", "Arc<Database>", &["config", "config"]);
    db.constructor_name = "connect".to_string();
    db.is_async = true;

    let mut auth = make_dep("auth", "Arc<AuthService>", &["config", "db"]);
    auth.custom_builder = Some("build_auth".to_string());

    let mut controller = make_dep("controller", "UserController", &["auth"]);
    controller.lifecycle = Lifecycle::Transient;

    let sorted = vec![config, db, auth, controller];
    let plan = ExecutionPlanner::new()
        .build_plan(&container, sorted)
        .unwrap();

    assert_eq!(plan.container_name, container_name("AppContainer"));
    assert!(plan.is_async);
    assert_eq!(plan.ordered_steps.len(), 4);
    assert_eq!(
        plan.ordered_steps[0].constructor_call,
        "Config<Prod>::new()"
    );
    assert_eq!(
        plan.ordered_steps[1].constructor_call,
        "Database::connect(config.clone(), config.clone())"
    );
    assert!(plan.ordered_steps[1].is_async);
    assert_eq!(
        plan.ordered_steps[2].constructor_call,
        "AppContainer::build_auth(config.clone(), db.clone())"
    );
    assert_eq!(
        plan.ordered_steps[3].constructor_call,
        "UserController::new(auth.clone())"
    );
    assert_eq!(plan.ordered_steps[3].lifecycle, Lifecycle::Transient);
}

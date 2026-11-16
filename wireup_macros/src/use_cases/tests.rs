use super::*;
use crate::domain::{
    ContainerIR, ContainerName, DependencyIR, DependencyName, DomainError, Lifecycle, TypeName,
};

fn dep_name(s: &str) -> DependencyName {
    DependencyName::new(s).expect("valid dep name")
}

fn type_name(s: &str) -> TypeName {
    TypeName::new(s).expect("valid type name")
}

fn container_name(s: &str) -> ContainerName {
    ContainerName::new(s).expect("valid container name")
}

fn make_dep(name: &str, ty: &str, prereqs: &[&str]) -> DependencyIR {
    let mut dep = DependencyIR::new(dep_name(name), type_name(ty));
    dep.prerequisites = prereqs.iter().map(|p| dep_name(p)).collect();
    dep
}

fn assert_precedes(order: &[DependencyName], earlier: &str, later: &str) {
    let e_pos = order.iter().position(|d| d.as_str() == earlier).unwrap();
    let l_pos = order.iter().position(|d| d.as_str() == later).unwrap();
    assert!(e_pos < l_pos, "expected '{earlier}' before '{later}'");
}

#[test]
fn test_linear_graph_topological_sort() {
    let deps = vec![
        make_dep("controller", "UserController", &["service"]),
        make_dep("service", "Arc<UserService>", &["db"]),
        make_dep("db", "Arc<Database>", &["config"]),
        make_dep("config", "Arc<Config>", &[]),
    ];
    let sorted = GraphEngine::new().topological_sort(&deps).unwrap();
    let names: Vec<DependencyName> = sorted.into_iter().map(|d| d.name).collect();
    assert_eq!(
        names,
        vec![
            dep_name("config"),
            dep_name("db"),
            dep_name("service"),
            dep_name("controller")
        ]
    );
}

#[test]
fn test_diamond_graph_topological_sort() {
    let deps = vec![
        make_dep("api", "ApiRouter", &["user_service", "order_service"]),
        make_dep("user_service", "Arc<UserService>", &["db"]),
        make_dep("order_service", "Arc<OrderService>", &["db"]),
        make_dep("db", "Arc<Database>", &["config"]),
        make_dep("config", "Arc<Config>", &[]),
    ];
    let sorted = GraphEngine::new().topological_sort(&deps).unwrap();
    let names: Vec<DependencyName> = sorted.into_iter().map(|d| d.name).collect();

    assert_eq!(names.len(), 5);
    assert_precedes(&names, "config", "db");
    assert_precedes(&names, "db", "user_service");
    assert_precedes(&names, "db", "order_service");
    assert_precedes(&names, "user_service", "api");
    assert_precedes(&names, "order_service", "api");
}

#[test]
fn test_disconnected_graph_topological_sort() {
    let deps = vec![
        make_dep("service_a", "Arc<ServiceA>", &["config_a"]),
        make_dep("config_a", "Arc<ConfigA>", &[]),
        make_dep("service_b", "Arc<ServiceB>", &["config_b"]),
        make_dep("config_b", "Arc<ConfigB>", &[]),
        make_dep("standalone", "Arc<Metrics>", &[]),
    ];
    let sorted = GraphEngine::new().topological_sort(&deps).unwrap();
    let names: Vec<DependencyName> = sorted.into_iter().map(|d| d.name).collect();

    assert_eq!(names.len(), 5);
    assert_precedes(&names, "config_a", "service_a");
    assert_precedes(&names, "config_b", "service_b");
    assert!(names.contains(&dep_name("standalone")));
}

#[test]
fn test_self_cycle_detection() {
    let deps = vec![
        make_dep("config", "Arc<Config>", &[]),
        make_dep("recursive", "Arc<Recursive>", &["recursive"]),
    ];
    let err = GraphEngine::new().topological_sort(&deps).unwrap_err();
    assert_eq!(
        err,
        DomainError::CycleDetected(vec![dep_name("recursive"), dep_name("recursive")])
    );
}

#[test]
fn test_indirect_cycle_detection() {
    let deps = vec![
        make_dep("config", "Arc<Config>", &[]),
        make_dep("service_a", "Arc<ServiceA>", &["config", "service_b"]),
        make_dep("service_b", "Arc<ServiceB>", &["service_c"]),
        make_dep("service_c", "Arc<ServiceC>", &["service_a"]),
    ];
    let err = GraphEngine::new().topological_sort(&deps).unwrap_err();
    assert_eq!(
        err,
        DomainError::CycleDetected(vec![
            dep_name("service_a"),
            dep_name("service_b"),
            dep_name("service_c"),
            dep_name("service_a"),
        ])
    );
}

#[test]
fn test_missing_prerequisites_and_duplicate_validation() {
    let engine = GraphEngine::new();
    let missing = vec![make_dep(
        "user_service",
        "Arc<UserService>",
        &["db", "cache"],
    )];
    assert_eq!(
        engine.topological_sort(&missing).unwrap_err(),
        DomainError::MissingPrerequisites {
            dependency: dep_name("user_service"),
            missing: vec![dep_name("db"), dep_name("cache")],
        }
    );

    let dup = vec![
        make_dep("db", "Arc<PostgresDb>", &[]),
        make_dep("db", "Arc<SqliteDb>", &[]),
    ];
    assert_eq!(
        engine.topological_sort(&dup).unwrap_err(),
        DomainError::DuplicateDependency(dep_name("db"))
    );
}

#[test]
fn test_derive_container_use_case_end_to_end() {
    let mut container = ContainerIR::new(container_name("AppContainer"));
    let config = make_dep("config", "Arc<Config>", &[]);
    let mut db = make_dep("db", "Arc<Database>", &["config"]);
    db.constructor_name = "connect".to_string();
    db.is_async = true;

    let mut auth = make_dep("auth", "Arc<AuthService>", &["config", "db"]);
    auth.custom_builder = Some("build_auth".to_string());

    let mut controller = make_dep("controller", "UserController", &["auth"]);
    controller.lifecycle = Lifecycle::Transient;

    container.dependencies = vec![controller, auth, db, config];
    let plan = DeriveContainerUseCase::new().execute(container).unwrap();

    assert_eq!(plan.container_name, container_name("AppContainer"));
    assert_eq!(plan.key_enum_name, "AppContainerKey");
    assert!(plan.is_async);
    assert_eq!(plan.ordered_steps.len(), 4);
    assert_eq!(plan.ordered_steps[0].constructor_call, "Config::new()");
    assert_eq!(
        plan.ordered_steps[1].constructor_call,
        "Database::connect(config.clone())"
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

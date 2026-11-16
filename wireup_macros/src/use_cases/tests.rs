use super::*;
use crate::domain::{
    ContainerIR, ContainerName, DependencyIR, DependencyName, DomainError, Lifecycle, TypeName,
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

fn assert_precedes(order: &[DependencyName], earlier: &str, later: &str) {
    let e_pos = order.iter().position(|d| d.as_str() == earlier).unwrap();
    let l_pos = order.iter().position(|d| d.as_str() == later).unwrap();
    assert!(e_pos < l_pos, "expected '{earlier}' before '{later}'");
}

#[test]
fn test_empty_and_linear_graph_topological_sort() {
    let engine = GraphEngine::new();
    assert!(engine.topological_sort(&[]).unwrap().is_empty());

    let deps = vec![
        make_dep("controller", "UserController", &["service"]),
        make_dep("service", "Arc<UserService>", &["db"]),
        make_dep("db", "Arc<Database>", &["config"]),
        make_dep("config", "Arc<Config>", &[]),
    ];
    let sorted = engine.topological_sort(&deps).unwrap();
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
fn test_diamond_and_disconnected_graphs() {
    let engine = GraphEngine::new();
    let diamond = vec![
        make_dep("api", "ApiRouter", &["user_service", "order_service"]),
        make_dep("user_service", "Arc<UserService>", &["db"]),
        make_dep("order_service", "Arc<OrderService>", &["db"]),
        make_dep("db", "Arc<Database>", &["config"]),
        make_dep("config", "Arc<Config>", &[]),
    ];
    let names: Vec<DependencyName> = engine
        .topological_sort(&diamond)
        .unwrap()
        .into_iter()
        .map(|d| d.name)
        .collect();
    assert_precedes(&names, "config", "db");
    assert_precedes(&names, "db", "user_service");
    assert_precedes(&names, "db", "order_service");
    assert_precedes(&names, "user_service", "api");
    assert_precedes(&names, "order_service", "api");

    let disconnected = vec![
        make_dep("service_a", "Arc<ServiceA>", &["config_a"]),
        make_dep("config_a", "Arc<ConfigA>", &[]),
        make_dep("standalone", "Arc<Metrics>", &[]),
    ];
    let disc_names: Vec<DependencyName> = engine
        .topological_sort(&disconnected)
        .unwrap()
        .into_iter()
        .map(|d| d.name)
        .collect();
    assert_precedes(&disc_names, "config_a", "service_a");
    assert!(disc_names.contains(&dep_name("standalone")));
}

#[test]
fn test_self_and_indirect_cycle_with_non_cycle_neighbors() {
    let engine = GraphEngine::new();
    let self_cycle = vec![make_dep("recursive", "Arc<Recursive>", &["recursive"])];
    assert_eq!(
        engine.topological_sort(&self_cycle).unwrap_err(),
        DomainError::CycleDetected(vec![dep_name("recursive"), dep_name("recursive")])
    );

    let indirect = vec![
        make_dep("downstream", "Arc<Downstream>", &["service_a"]),
        make_dep("leaf_config", "Arc<Config>", &[]),
        make_dep("service_a", "Arc<ServiceA>", &["leaf_config", "service_b"]),
        make_dep("service_b", "Arc<ServiceB>", &["service_c"]),
        make_dep("service_c", "Arc<ServiceC>", &["service_a"]),
    ];
    assert_eq!(
        engine.topological_sort(&indirect).unwrap_err(),
        DomainError::CycleDetected(vec![
            dep_name("service_a"),
            dep_name("service_b"),
            dep_name("service_c"),
            dep_name("service_a"),
        ])
    );
}

#[test]
fn test_figure_eight_scc_backtracking_cycle_detection() {
    // 'a -> b -> c -> b' explores a sub-cycle before backtracking through 'd -> a'
    let deps = vec![
        make_dep("a", "Arc<A>", &["b"]),
        make_dep("b", "Arc<B>", &["c", "d"]),
        make_dep("c", "Arc<C>", &["b"]),
        make_dep("d", "Arc<D>", &["a"]),
    ];
    let err = GraphEngine::new().topological_sort(&deps).unwrap_err();
    assert_eq!(
        err,
        DomainError::CycleDetected(vec![
            dep_name("a"),
            dep_name("b"),
            dep_name("d"),
            dep_name("a")
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
fn test_derive_container_and_edge_case_planning() {
    let mut container = ContainerIR::new(container_name("AppContainer"));
    let config = make_dep("config", "std::sync::Arc<  Config<Prod>  >", &[]);
    let mut db = make_dep("db", "Arc<Database>", &["config", "config"]);
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
    assert_eq!(
        plan.ordered_steps[0].constructor_call,
        "Config<Prod>::new()"
    );
    assert_eq!(
        plan.ordered_steps[1].constructor_call,
        "Database::connect(config.clone(), config.clone())"
    );
    assert_eq!(
        plan.ordered_steps[2].constructor_call,
        "AppContainer::build_auth(config.clone(), db.clone())"
    );
    assert_eq!(plan.ordered_steps[3].lifecycle, Lifecycle::Transient);
}

use super::*;
use crate::domain::{ContainerName, DependencyName, Lifecycle, TypeName};

fn make_step(
    name: &str,
    ty: &str,
    call: &str,
    is_async: bool,
    lifecycle: Lifecycle,
) -> InstantiationStep {
    InstantiationStep::new(
        DependencyName::new(name).unwrap(),
        TypeName::new(ty).unwrap(),
        call,
        is_async,
        lifecycle,
    )
}

#[test]
fn test_emit_sync_struct_strips_transients_and_wraps_arc_singletons() {
    let mut plan = ConstructionPlan::new(
        ContainerName::new("AppContainer").unwrap(),
        false,
        "AppContainerKey",
    );
    plan.ordered_steps = vec![
        make_step(
            "config",
            "Arc<Config>",
            "Config::new()",
            false,
            Lifecycle::Singleton,
        ),
        make_step(
            "auth",
            "Arc<AuthService>",
            "AppContainer::build_auth(config.clone())",
            false,
            Lifecycle::Singleton,
        ),
        make_step(
            "controller",
            "UserController",
            "UserController::new(auth.clone())",
            false,
            Lifecycle::Transient,
        ),
    ];

    let code = emit_struct_and_init(&plan).to_string();

    assert!(code.contains(
        "pub struct AppContainer { config : Arc < Config > , auth : Arc < AuthService > , }"
    ));
    assert!(!code.contains("controller : UserController"));
    assert!(code.contains("pub fn new () -> Self"));
    assert!(code.contains(
        "let config : Arc < Config > = :: std :: sync :: Arc :: new (Config :: new ()) ;"
    ));
    assert!(code.contains(
        "let auth : Arc < AuthService > = AppContainer :: build_auth (config . clone ()) ;"
    ));
    assert!(!code.contains("let controller"));
}

#[test]
fn test_emit_async_init_supports_sequential_and_parallel_batches() {
    let mut plan =
        ConstructionPlan::new(ContainerName::new("AsyncApp").unwrap(), true, "AsyncAppKey");
    plan.ordered_steps = vec![
        make_step(
            "config",
            "Arc<Config>",
            "Config::new()",
            false,
            Lifecycle::Singleton,
        ),
        make_step(
            "db",
            "Arc<Database>",
            "Database::connect(config.clone())",
            true,
            Lifecycle::Singleton,
        ),
        make_step(
            "payment",
            "Arc<PaymentGateway>",
            "PaymentGateway::init(config.clone())",
            true,
            Lifecycle::Singleton,
        ),
    ];
    plan.parallel_batches = vec![vec![
        DependencyName::new("db").unwrap(),
        DependencyName::new("payment").unwrap(),
    ]];

    let code = emit_struct_and_init(&plan).to_string();

    assert!(code.contains("pub async fn init ()"));
    assert!(code.contains(":: tokio :: join !"));
    assert!(code.contains("let (db_res , payment_res)"));
    assert!(code.contains("let db : Arc < Database > = :: std :: sync :: Arc :: new (db_res ?) ;"));
}

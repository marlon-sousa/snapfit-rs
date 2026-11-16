use super::*;
use crate::domain::{ContainerName, DependencyName, Lifecycle, TypeName};

fn make_step(name: &str, ty: &str, call: &str, lifecycle: Lifecycle) -> InstantiationStep {
    InstantiationStep::new(
        DependencyName::new(name).unwrap(),
        TypeName::new(ty).unwrap(),
        call,
        false,
        lifecycle,
    )
}

#[test]
fn test_emit_getters_generates_singleton_and_transient_getters() {
    let mut plan = ConstructionPlan::new(
        ContainerName::new("AppContainer").unwrap(),
        false,
        "AppContainerKey",
    );
    plan.ordered_steps = vec![
        make_step(
            "db",
            "Arc<Database>",
            "Database::new()",
            Lifecycle::Singleton,
        ),
        make_step(
            "controller",
            "UserController",
            "UserController::new(db.clone())",
            Lifecycle::Transient,
        ),
    ];

    let code = emit_getters_impl(&plan).to_string();

    assert!(code.contains("pub fn get_db (& self) -> Arc < Database > { self . db . clone () }"));
    assert!(code.contains("pub fn get_controller (& self) -> UserController { let db = self . get_db () ; UserController :: new (db . clone ()) }"));
    assert!(code.contains("pub fn get_dependency < T : 'static + Clone >"));
    assert!(code.contains(
        "AppContainerKey :: Db => self . downcast_dep :: < Arc < Database > , T > (self . get_db () , key_str)"
    ));
}

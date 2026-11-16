use super::*;
use crate::domain::{ContainerName, DependencyName, Lifecycle, TypeName};

fn make_step(name: &str, ty: &str) -> InstantiationStep {
    InstantiationStep::new(
        DependencyName::new(name).unwrap(),
        TypeName::new(ty).unwrap(),
        format!("{ty}::new()"),
        false,
        Lifecycle::Singleton,
    )
}

#[test]
fn test_emit_key_enum_generates_pascal_case_variants_and_mappings() {
    let mut plan = ConstructionPlan::new(
        ContainerName::new("AppContainer").unwrap(),
        false,
        "AppContainerKey",
    );
    plan.ordered_steps = vec![
        make_step("config", "Arc<Config>"),
        make_step("user_controller", "UserController"),
    ];

    let code = emit_key_enum(&plan).to_string();

    assert!(code.contains("pub enum AppContainerKey"));
    assert!(code.contains("Config ,"));
    assert!(code.contains("UserController ,"));
    assert!(code.contains("Self :: Config => \"config\""));
    assert!(code.contains("Self :: UserController => \"user_controller\""));
    assert!(code.contains("\"config\" => :: std :: result :: Result :: Ok (Self :: Config)"));
    assert!(code.contains(
        "\"user_controller\" => :: std :: result :: Result :: Ok (Self :: UserController)"
    ));
    assert!(code.contains("UnknownKey"));
}

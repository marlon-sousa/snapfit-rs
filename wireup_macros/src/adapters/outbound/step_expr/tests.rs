use super::*;
use crate::domain::{ContainerName, DependencyName, Lifecycle, TypeName};

fn make_step(name: &str, ty: &str, call: &str, is_async: bool) -> InstantiationStep {
    InstantiationStep::new(
        DependencyName::new(name).unwrap(),
        TypeName::new(ty).unwrap(),
        call,
        is_async,
        Lifecycle::Singleton,
    )
}

#[test]
fn test_references_dependency_matches_exact_argument_ident() {
    let call = "Repo::new(read_db.clone(), write_db.clone())";
    assert!(references_dependency(call, "read_db"));
    assert!(references_dependency(call, "write_db"));
    assert!(!references_dependency(call, "db"));
}

#[test]
fn test_build_call_expr_wraps_standard_arc_and_preserves_custom_builder() {
    let plan = ConstructionPlan::new(
        ContainerName::new("AppContainer").unwrap(),
        true,
        "AppContainerKey",
    );
    let standard_step = make_step("db", "Arc<Database>", "Database::connect()", true);
    let builder_step = make_step(
        "auth",
        "Arc<AuthService>",
        "AppContainer::build_auth()",
        false,
    );

    assert_eq!(
        build_call_expr(&plan, &standard_step).to_string(),
        ":: std :: sync :: Arc :: new (Database :: connect () . await ?)"
    );
    assert_eq!(
        build_call_expr(&plan, &builder_step).to_string(),
        "AppContainer :: build_auth ()"
    );
}

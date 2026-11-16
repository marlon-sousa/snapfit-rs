use super::*;
use crate::domain::{ContainerIR, ContainerName, DependencyIR, DependencyName, TypeName};

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
fn test_derive_container_use_case_orchestrates_sorting_and_planning() {
    let mut container = ContainerIR::new(container_name("AppContainer"));
    let config = make_dep("config", "Arc<Config>", &[]);
    let db = make_dep("db", "Arc<Database>", &["config"]);
    let service = make_dep("service", "Arc<UserService>", &["db"]);

    container.dependencies = vec![service, db, config];

    let use_case =
        DeriveContainerUseCase::with_components(GraphEngine::new(), ExecutionPlanner::new());
    let plan = use_case.execute(container).unwrap();

    let step_names: Vec<DependencyName> =
        plan.ordered_steps.into_iter().map(|s| s.dep_name).collect();
    assert_eq!(
        step_names,
        vec![dep_name("config"), dep_name("db"), dep_name("service")]
    );
}

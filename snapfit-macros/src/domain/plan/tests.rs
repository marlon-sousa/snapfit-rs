use super::*;

fn step(name: &str, lifecycle: Lifecycle) -> Rc<Step<()>> {
    let call = Call::Constructor {
        constructor: Constructor::New,
        wrapping: Wrapping::None,
    };
    let name = DependencyName::new(name).unwrap();
    Rc::new(Step::new(
        name,
        (),
        lifecycle,
        Execution::Sync,
        Fallibility::Infallible,
        call,
        Vec::new(),
    ))
}

#[test]
fn singletons_are_listed_in_construction_order_without_transients() {
    let config = step("config", Lifecycle::Singleton);
    let request = step("request", Lifecycle::Transient);
    let db = step("db", Lifecycle::Singleton);
    let order = vec![config, request, db];
    let plan = ConstructionPlan::new(
        (),
        Execution::Sync,
        Fallibility::Infallible,
        order.clone(),
        order,
        Vec::<DispatchedTypePlan<(), ()>>::new(),
    );

    let names: Vec<_> = plan.singletons().map(|step| step.name.as_str()).collect();

    assert_eq!(names, ["config", "db"]);
}

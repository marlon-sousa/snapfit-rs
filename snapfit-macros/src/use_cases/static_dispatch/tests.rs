use std::iter;

use super::*;
use crate::test_support::dispatched_types::{
    declared, dispatched_field, dispatching_container, extended, type_name, variant, written_name,
};

fn plan_names(plan: &DispatchedTypePlan<&'static str, &'static str>) -> Vec<&'static str> {
    iter::once(&plan.first)
        .chain(&plan.rest)
        .map(|variant| variant.syntax)
        .collect()
}

#[test]
fn a_dispatched_type_holds_its_fields_then_its_extra_then_its_extensions_each_once() {
    let container = dispatching_container(
        vec![
            dispatched_field("main", written_name("Db"), Some("MySql")),
            dispatched_field("other", written_name("Cache"), Some("Redis")),
            dispatched_field("backup", written_name("Db"), Some("Sqlite")),
            dispatched_field("again", written_name("Db"), Some("MySql")),
        ],
        vec![declared("Db", 0, &["Fake", "Sqlite"])],
        vec![
            extended("Db", 1, &["Mock", "Fake"]),
            extended("Cache", 2, &["Lost"]),
        ],
    );

    let plans = dispatched_types(&container);

    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].syntax, "Db");
    assert_eq!(plan_names(&plans[0]), ["MySql", "Sqlite", "Fake", "Mock"]);
}

#[test]
fn a_repeated_declaration_is_planned_once_under_the_first() {
    let container = dispatching_container(
        Vec::new(),
        vec![
            declared("Db", 0, &["MySql"]),
            declared("Db", 1, &["Sqlite"]),
        ],
        Vec::new(),
    );

    let plans = dispatched_types(&container);

    assert_eq!(plans.len(), 1);
    assert_eq!(plan_names(&plans[0]), ["MySql"]);
}

#[test]
fn a_field_holds_a_declared_name_written_alone_or_through_a_path() {
    let names: DispatchedNames = [declared("Db", 0, &[])].into_iter().collect();
    let path = WrittenType::Path {
        last: type_name("Db"),
        written: "shop::Db".to_owned(),
    };

    assert_eq!(
        names.held_by(&dispatched_field("a", written_name("Db"), None)),
        Some(&type_name("Db"))
    );
    assert_eq!(
        names.held_by(&dispatched_field("b", path, None)),
        Some(&type_name("Db"))
    );
    assert_eq!(
        names.held_by(&dispatched_field("c", written_name("Cache"), None)),
        None
    );
    assert_eq!(
        names.held_by(&dispatched_field("d", WrittenType::Other, None)),
        None
    );
    assert_eq!(dispatch_of(Some(&type_name("Db"))), Dispatch::Static);
    assert_eq!(dispatch_of(None), Dispatch::None);
}

#[test]
fn a_refused_container_falls_back_to_what_its_readable_parts_give_each_type() {
    let fallbacks = fallback_dispatched_types(
        &[
            declared("Db", 0, &["Fake"]),
            declared("Cache", 1, &[]),
            declared("Db", 2, &[]),
        ],
        &[extended("Db", 3, &["Mock"])],
        &[dispatched_field("main", written_name("Db"), Some("MySql"))],
    );

    assert_eq!(
        fallbacks,
        [
            DispatchedTypeFallback::Planned(DispatchedTypePlan::new(
                "Db",
                variant("MySql"),
                vec![variant("Fake"), variant("Mock")],
            )),
            DispatchedTypeFallback::Unplanned("Cache"),
        ]
    );
}

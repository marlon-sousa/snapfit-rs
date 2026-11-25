use std::iter;

use super::*;
use crate::domain::Respelling;
use crate::test_support::{
    declared, dispatched_field, dispatching_container, extended, listing_field, type_name, variant,
    written_name,
};

fn plan_names(plan: &DispatchedTypePlan<&'static str, &'static str>) -> Vec<&'static str> {
    iter::once(&plan.first)
        .chain(&plan.rest)
        .map(|variant| variant.syntax)
        .collect()
}

#[test]
fn a_dispatched_type_holds_its_fields_lists_included_then_its_extensions_each_once() {
    let container = dispatching_container(
        vec![
            dispatched_field("main", written_name("Db"), Some("MySql")),
            dispatched_field("other", written_name("Cache"), Some("Redis")),
            dispatched_field("backup", written_name("Db"), Some("Sqlite")),
            listing_field("chosen", written_name("Db"), &["Fake", "Sqlite"]),
            dispatched_field("again", written_name("Db"), Some("MySql")),
        ],
        vec![declared("Db", 0)],
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
fn a_type_written_with_self_is_held_once_beside_the_same_type_written_without_it() {
    let container = dispatching_container(
        vec![
            dispatched_field("main", written_name("Db"), Some("Mock")),
            listing_field(
                "chosen",
                written_name("Db"),
                &["self::Mock", "crate::db::Mock"],
            ),
        ],
        vec![declared("Db", 0)],
        vec![extended("Db", 1, &["Cache<self::Order>", "Cache<Order>"])],
    );

    let plans = dispatched_types(&container);

    assert_eq!(
        plan_names(&plans[0]),
        ["Mock", "crate::db::Mock", "Cache<self::Order>"]
    );
    assert_eq!(
        plans[0].respelled,
        [
            Respelling::new("Mock", "self::Mock"),
            Respelling::new("Cache<self::Order>", "Cache<Order>"),
        ]
    );
}

#[test]
fn a_type_named_again_as_held_is_no_other_spelling_but_each_other_spelling_is_checked_where_written()
 {
    let container = dispatching_container(
        vec![
            dispatched_field("main", written_name("Db"), Some("Mock")),
            listing_field(
                "chosen",
                written_name("Db"),
                &["Mock", "self::Mock", "self::Mock"],
            ),
        ],
        vec![declared("Db", 0)],
        vec![],
    );

    let plans = dispatched_types(&container);

    assert_eq!(plan_names(&plans[0]), ["Mock"]);
    assert_eq!(
        plans[0].respelled,
        [
            Respelling::new("Mock", "self::Mock"),
            Respelling::new("Mock", "self::Mock"),
        ]
    );
}

#[test]
fn a_repeated_declaration_is_planned_once_under_the_first() {
    let container = dispatching_container(
        vec![
            listing_field("main", written_name("Db"), &["MySql"]),
            listing_field("other", written_name("Db"), &["Sqlite"]),
        ],
        vec![declared("Db", 0), declared("Db", 1)],
        Vec::new(),
    );

    let plans = dispatched_types(&container);

    assert_eq!(plans.len(), 1);
    assert_eq!(plan_names(&plans[0]), ["MySql", "Sqlite"]);
}

#[test]
fn a_field_holds_a_declared_name_written_alone_or_through_a_path() {
    let names: DispatchedNames = [declared("Db", 0)].into_iter().collect();
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
        &[declared("Db", 0), declared("Cache", 1), declared("Db", 2)],
        &[extended("Db", 3, &["Mock"])],
        &[
            dispatched_field("main", written_name("Db"), Some("MySql")),
            listing_field("chosen", written_name("Db"), &["Fake"]),
        ],
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

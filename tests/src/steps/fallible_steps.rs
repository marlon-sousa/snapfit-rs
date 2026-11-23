use std::error::Error;
use std::sync::Arc;

use cucumber::{then, when};

use crate::fallible_containers::outage::PortTaken;
use crate::world::ContainerWorld;

#[when(
    "the shop whose configuration, tax rates, database and payment gateway can each fail is opened and none fails"
)]
fn when_shop_opened(world: &mut ContainerWorld) {
    world.built.fallible.open_shop();
}

#[then("every field should be built from the value its constructor, builder or factory returned")]
fn then_every_field_built_from_its_value(world: &mut ContainerWorld) {
    let shop = world.built.fallible.shop();
    let orders = shop.get_orders();

    assert_eq!(shop.get_config().currency, "EUR");
    assert_eq!(shop.get_rates().standard_percent, 20);
    assert!(Arc::ptr_eq(&shop.get_db().config, &shop.get_config()));
    assert!(Arc::ptr_eq(&orders.db, &shop.get_db()));
    assert_eq!(orders.payments.name(), "stripe");
}

#[then(
    "its receipt printer should be built by Default, and its tax rates, a fallible leaf with a build of its own, by their own new"
)]
fn then_a_fallible_leaf_with_its_own_build_is_built_by_it(world: &mut ContainerWorld) {
    let shop = world.built.fallible.shop();

    assert_eq!(shop.get_receipts().paper, "");
    assert_eq!(shop.get_rates().standard_percent, 20);
}

#[when("the till whose leaf_build is written constructor(new) and whose ledger can fail is opened")]
fn when_till_opened(world: &mut ContainerWorld) {
    world.built.fallible.open_till();
}

#[then("its ledger should be built by the ledger's own new")]
fn then_ledger_built_by_its_own_new(world: &mut ContainerWorld) {
    assert_eq!(world.built.fallible.till().get_ledger().entries, 0);
}

#[when("the storefront whose database fails to connect after its configuration is built is opened")]
fn when_storefront_opened(world: &mut ContainerWorld) {
    world.built.fallible.open_storefront();
}

#[then("the error should say that snapfit could not build the field db of Storefront")]
fn then_error_names_field_and_container(world: &mut ContainerWorld) {
    let failure = world.built.fallible.failure();

    assert_eq!(
        failure.to_string(),
        "snapfit could not build the field `db` of `Storefront`"
    );
    assert_eq!(failure.container(), "Storefront");
    assert_eq!(failure.field(), "db");
}

#[then("its source should be the database's own error, saying that port 5432 is taken")]
fn then_source_is_the_field_s_own_error(world: &mut ContainerWorld) {
    let source = world
        .built
        .fallible
        .failure()
        .source()
        .and_then(|source| source.downcast_ref::<PortTaken>());

    assert_eq!(source, Some(&PortTaken(5432)));
}

#[then("the configuration should have been built and dropped, and the order service never built")]
fn then_build_stopped_at_first_failure(world: &mut ContainerWorld) {
    assert_eq!(
        world.built.fallible.events(),
        ["config built", "database tried", "config dropped"]
    );
}

#[when("the tills that a macro_rules! writes from the tokens it is passed are opened")]
fn when_macro_tills_opened(world: &mut ContainerWorld) {
    world.built.fallible.open_macro_tills();
}

#[then(expr = "the till passed {string} should have built its ledger by the ledger's own new")]
fn then_passed_till_built_its_ledger(world: &mut ContainerWorld, form: String) {
    assert_eq!(world.built.fallible.passed_ledger_entries(&form), 0);
}

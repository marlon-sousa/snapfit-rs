use std::sync::Arc;

use cucumber::{then, when};

use crate::world::ContainerWorld;

#[when(
    "the books whose tax rates are built by Default and whose ledger is opened by its open constructor are instantiated"
)]
fn when_books_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_books();
}

#[then("the ledger should be opened by open with the container's own default tax rates")]
fn then_ledger_opened_with_default_rates(world: &mut ContainerWorld) {
    let books = world.built.books();
    let ledger = books.get_ledger();

    assert_eq!(ledger.opened_by, "open");
    assert_eq!(ledger.rates.standard_percent, 0);
    assert!(Arc::ptr_eq(&ledger.rates, &books.get_rates()));
}

#[when(
    "the shop whose order store is built by a factory from write_db and read_db is instantiated"
)]
fn when_shop_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_shop();
}

#[then("the order store should read from the container's read_db and write to its write_db")]
fn then_order_store_gets_each_database_by_name(world: &mut ContainerWorld) {
    let shop = world.built.shop();
    let store = shop.get_order_store();

    assert!(
        Arc::ptr_eq(store.reader(), &shop.get_read_db()),
        "Expected the order store to read from read_db"
    );
    assert!(
        Arc::ptr_eq(store.writer(), &shop.get_write_db()),
        "Expected the order store to write to write_db"
    );
    assert!(!Arc::ptr_eq(store.reader(), store.writer()));
}

#[then("the order store should be built once and shared")]
fn then_order_store_is_a_singleton(world: &mut ContainerWorld) {
    let shop = world.built.shop();

    assert!(Arc::ptr_eq(
        &shop.get_order_store(),
        &shop.get_order_store()
    ));
}

#[when("the store whose trait-object payments field is built by a factory is instantiated")]
fn when_store_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_store();
}

#[then(expr = "the checkout should charge through the factory's gateway, reading {string}")]
fn then_checkout_charges_through_factory_gateway(world: &mut ContainerWorld, charged: String) {
    let store = world.built.store();
    let checkout = store.get_checkout();

    assert_eq!(checkout.gateway().charge(250), charged);
    assert!(Arc::ptr_eq(checkout.gateway(), &store.get_payments()));
}

#[when("the billing container whose leaves are built by Default is instantiated")]
fn when_billing_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_billing();
}

#[then(
    "its leaves should be built by Default, its mailer by its own constructor, and its invoicer by new"
)]
fn then_leaves_built_by_default(world: &mut ContainerWorld) {
    let billing = world.built.billing();

    assert_eq!(billing.get_settings().currency, "");
    assert_eq!(billing.get_mailer().sender, "sandbox");
    assert!(Arc::ptr_eq(
        &billing.get_invoicer().settings,
        &billing.get_settings()
    ));
    // The clock has no new, so the container compiling proves Default built it.
    let _clock = billing.get_clock();
}

#[when("the archive container whose leaf_build is constructor(open) is instantiated")]
fn when_archive_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_archive();
}

#[then("its leaves should be opened by open, and its leaf that says build = default should not be")]
fn then_leaves_opened_by_open(world: &mut ContainerWorld) {
    let archive = world.built.archive();

    assert_eq!(archive.get_books().opened_by, "open");
    assert_eq!(archive.get_maps().opened_by, "open");
    assert_eq!(archive.get_spare().opened_by, "default");
}

#[when(
    "the office declared by a macro_rules! that passes its leaf_build, deps and constructor as expressions is instantiated"
)]
fn when_office_instantiated(world: &mut ContainerWorld) {
    world.built.instantiate_office();
}

#[then("its settings should be built by Default and its mailer connected to them")]
fn then_office_read_as_written_by_hand(world: &mut ContainerWorld) {
    let office = world.built.office();
    let mailer = office.get_mailer();

    assert_eq!(office.get_settings().region, "");
    assert_eq!(mailer.opened_by, "connect");
    assert!(Arc::ptr_eq(&mailer.settings, &office.get_settings()));
}

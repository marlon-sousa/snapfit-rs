use cucumber::{given, then, when};
use snapfit::__private::{Either, Only};

use crate::dispatch_world::{DatabaseKind, StoreKind};
use crate::dispatched_traits::conditional::{Bulb, Lamp, Lamps};
use crate::dispatched_traits::databases::{Database, Mock, Named};
use crate::dispatched_traits::stores::Store;
use crate::dispatched_traits::written_by_a_macro::{Oak, Rack, Racks};
use crate::world::ContainerWorld;

static ITEMS: [u32; 2] = [1, 2];

#[given(expr = "a database dispatched statically, holding {database}")]
fn given_dispatched_database(world: &mut ContainerWorld, database: DatabaseKind) {
    world.built.dispatch.database = Some(database.dispatched());
}

#[then(expr = "finding order {int} should answer from {database}")]
fn then_finding_answers_from(world: &mut ContainerWorld, id: u32, database: DatabaseKind) {
    let order = world.built.dispatch.database().find(id);
    assert_eq!(order.map(|order| order.source), Some(name_of(database)));
}

#[then(expr = "its name should be {database}")]
fn then_its_name_is(world: &mut ContainerWorld, database: DatabaseKind) {
    assert_eq!(world.built.dispatch.database().name(), name_of(database));
}

#[then(expr = "tagging it with {int} should read {string}")]
fn then_tagging_reads(world: &mut ContainerWorld, tag: u32, expected: String) {
    assert_eq!(world.built.dispatch.database().tagged(tag), expected);
}

#[then(expr = "showing {string} through it should read {string}")]
fn then_showing_reads(world: &mut ContainerWorld, value: String, expected: String) {
    assert_eq!(world.built.dispatch.database().shown(value), expected);
}

#[then(expr = "pinging it should read {string}")]
async fn then_pinging_reads(world: &mut ContainerWorld, expected: String) {
    assert_eq!(world.built.dispatch.database().ping().await, expected);
}

#[then(expr = "its greeting should read {string}")]
fn then_greeting_reads(world: &mut ContainerWorld, expected: String) {
    assert_eq!(world.built.dispatch.database().greeting(), expected);
}

#[when(expr = "it stores the orders {int} and {int}")]
fn when_it_stores(world: &mut ContainerWorld, first: u32, second: u32) {
    let database = world.built.dispatch.database_mut();
    database.store(first);
    database.store(second);
}

#[then(expr = "it should hold the orders {int} and {int}")]
fn then_it_holds(world: &mut ContainerWorld, first: u32, second: u32) {
    assert_eq!(world.built.dispatch.database().stored(), [first, second]);
}

#[then(expr = "taken by value, it should give its name {string}")]
fn then_by_value_gives(world: &mut ContainerWorld, expected: String) {
    let database = world.built.dispatch.database().clone();
    assert_eq!(database.into_name(), expected);
}

#[when(expr = "it is renamed with the prefix {string}")]
fn when_it_is_renamed(world: &mut ContainerWorld, prefix: String) {
    let database = world.built.dispatch.database.take();
    let prefix: &'static str = Box::leak(prefix.into_boxed_str());
    world.built.dispatch.database = database.map(|database| database.renamed(prefix));
}

#[then("it should still hold the mock")]
fn then_it_still_holds_the_mock(world: &mut ContainerWorld) {
    assert!(matches!(
        world.built.dispatch.database(),
        Either::Rest(Either::Rest(Only(Mock { .. })))
    ));
}

#[given(expr = "shelves dispatched statically, holding {store}")]
fn given_dispatched_shelves(world: &mut ContainerWorld, store: StoreKind) {
    world.built.dispatch.shelves = Some(store.dispatched());
}

#[when("two items are put on them")]
fn when_two_items_are_put(world: &mut ContainerWorld) {
    let runs = &mut world.built.dispatch;
    let shelves = runs
        .shelves
        .as_mut()
        .expect("dispatched shelves must be made in a Given step");
    runs.put = ITEMS.iter().map(|item| shelves.put(item)).last();
}

#[then(expr = "the last put should answer {int}")]
fn then_the_last_put_answers(world: &mut ContainerWorld, expected: usize) {
    assert_eq!(world.built.dispatch.put, Some(expected));
}

#[given("a rack dispatched statically, holding oak, whose trait a macro_rules! declares")]
fn given_macro_rack(world: &mut ContainerWorld) {
    world.built.dispatch.rack = Some(Racks::of(Oak));
}

#[then(expr = "the rack should read from {string}")]
fn then_rack_reads_from(world: &mut ContainerWorld, expected: String) {
    assert_eq!(world.built.dispatch.rack().source(), expected);
}

#[then("its copy should still hold oak")]
fn then_rack_copy_holds_oak(world: &mut ContainerWorld) {
    assert!(matches!(
        world.built.dispatch.rack().copied(),
        Either::First(Oak)
    ));
}

fn name_of(database: DatabaseKind) -> &'static str {
    database.into()
}

#[when(
    "a lamp whose trait a cfg_attr removes a method from is dispatched statically, holding a bulb"
)]
fn when_a_conditional_lamp_shines(world: &mut ContainerWorld) {
    world.built.dispatch.shone = Some(Lamps::of(Bulb).shine());
}

#[then(expr = "the lamp should shine {string}")]
fn then_the_lamp_shines(world: &mut ContainerWorld, expected: String) {
    assert_eq!(world.built.dispatch.shone, Some(expected.as_str()));
}

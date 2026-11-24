use cucumber::{then, when};

use crate::swap_world::{KeptForm, PassedForm, Source};
use crate::world::ContainerWorld;

#[when(expr = "the desk that reads from {source} is opened")]
fn when_desk_opened(world: &mut ContainerWorld, source: Source) {
    world.built.swaps.open_desk(source);
}

#[when("the desk that a macro_rules! writes, reading from disk, is opened")]
fn when_macro_desk_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_macro_desk();
}

#[then(expr = "its reader getter should return the trait object, reading from {source}")]
fn then_reader_getter_returns_the_trait(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.desk().reader, source);
}

#[then(expr = "its report, built by new, should read from {source}")]
fn then_report_reads_from(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.desk().report, source);
}

#[then(expr = "its digest, built by its builder, should read from {source}")]
fn then_digest_reads_from(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.desk().digest, source);
}

#[then(expr = "its summary, built by a factory from the trait object, should read from {source}")]
fn then_summary_reads_from(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.desk().summary, source);
}

#[when("the desks whose readers are built from their implementation in every way are opened")]
fn when_built_desks_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_built_desks();
}

#[then(expr = "the reader {word} should read from {string}")]
fn then_built_reader_reads_from(world: &mut ContainerWorld, reader: String, source: String) {
    let sources = &world.built.swaps.built().sources;
    let found = sources
        .iter()
        .find(|(name, _)| *name == reader)
        .unwrap_or_else(|| panic!("no reader is called {reader}"));
    assert_eq!(found.1, source);
}

#[then(
    expr = "the locked desk should fail at its field {word}, with the error {string} as its source"
)]
fn then_locked_desk_fails(world: &mut ContainerWorld, field: String, message: String) {
    let locked = &world.built.swaps.built().locked;
    assert_eq!(locked.field(), field);
    let source = std::error::Error::source(locked).map(ToString::to_string);
    assert_eq!(source, Some(message));
}

#[cfg(feature = "unstable")]
#[when("the desk whose transient reader names its implementation is opened")]
fn when_transient_desk_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_transient_desk();
}

#[cfg(feature = "unstable")]
#[then(expr = "every get of its reader should build a new trait object, reading from {source}")]
fn then_transient_reader_is_fresh(world: &mut ContainerWorld, source: Source) {
    let reads = world.built.swaps.transient();
    assert!(!reads.same_reader, "two gets returned the same reader");
    assert_eq!(reads.reader, source);
}

#[cfg(feature = "unstable")]
#[then(expr = "its report, built from a reader of its own, should read from {source}")]
fn then_transient_report_reads_from(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.transient().report, source);
}

#[when("the desk whose reader is neither Send nor Sync is opened")]
fn when_local_desk_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_local_desk();
}

#[then(expr = "its local reader should read from {source}")]
fn then_local_reader_reads_from(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.local(), source);
}

#[when("the desks whose implementation a macro_rules! passes as tokens are opened")]
fn when_passed_desks_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_passed_desks();
}

#[then(expr = "the desk whose implementation is passed as {passed_form} should read from {source}")]
fn then_passed_desk_reads_from(world: &mut ContainerWorld, form: PassedForm, source: Source) {
    assert_eq!(world.built.swaps.passed(form), source);
}

#[when("the desk whose field type a macro_rules! passes as a type is opened")]
fn when_typed_desk_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_typed_desk();
}

#[then(expr = "its reader, built from its implementation, should read from {source}")]
fn then_typed_reader_reads_from(world: &mut ContainerWorld, source: Source) {
    assert_eq!(world.built.swaps.typed(), source);
}

#[when("the desks whose services keep the reader's trait object inside their own type are opened")]
fn when_keeping_desks_opened(world: &mut ContainerWorld) {
    world.built.swaps.open_keeping_desks();
}

#[then(expr = "its service kept as {kept_form} should read from {source}")]
fn then_kept_service_reads_from(world: &mut ContainerWorld, form: KeptForm, source: Source) {
    assert_eq!(world.built.swaps.kept(form), source);
}

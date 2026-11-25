//! Steps that lint a fixture with clippy and judge what it reported.

use cucumber::{given, then, when};

use crate::world::ContainerWorld;

#[given(expr = "a container fixture {string} for clippy")]
fn given_clippy_fixture(world: &mut ContainerWorld, fixture_name: String) {
    world.select_fixture(fixture_name);
}

#[given(expr = "a dispatched trait fixture {string} for clippy")]
fn given_dispatched_trait_clippy_fixture(world: &mut ContainerWorld, fixture_name: String) {
    world.select_fixture(fixture_name);
}

#[given(expr = "a fixture {string} of a crate that depends on snapfit only under the name di")]
fn given_renamed_dependency_fixture(world: &mut ContainerWorld, fixture_name: String) {
    world.select_fixture(fixture_name);
}

#[when("that crate is linted with clippy")]
fn when_renamed_fixture_is_linted(world: &mut ContainerWorld) {
    world.lint_selected_renamed_fixture();
}

#[when("the fixture is linted with clippy")]
fn when_fixture_is_linted(world: &mut ContainerWorld) {
    world.lint_selected_fixture();
}

#[then("clippy should report no warning")]
fn then_clippy_reports_no_warning(world: &mut ContainerWorld) {
    let (fixture, outcome) = world.compiled_fixture();
    let reported: String = outcome
        .errors()
        .iter()
        .chain(outcome.warnings())
        .map(|diagnostic| diagnostic.rendered.as_str())
        .collect();
    assert!(
        outcome.is_compiled() && outcome.warnings().is_empty(),
        "Expected clippy to report nothing for fixture '{fixture}'. It said:\n{reported}"
    );
}

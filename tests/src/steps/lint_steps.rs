//! Steps that lint a fixture with clippy and judge what it reported.

use cucumber::{given, then, when};

use crate::world::ContainerWorld;

#[given(expr = "a container fixture {string} for clippy")]
fn given_clippy_fixture(world: &mut ContainerWorld, fixture_name: String) {
    world.select_fixture(fixture_name);
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

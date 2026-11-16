use cucumber::{given, then, when};

use crate::world::ContainerWorld;

#[given(expr = "an invalid container fixture {string}")]
fn given_invalid_container_fixture(world: &mut ContainerWorld, fixture_name: String) {
    world.select_invalid_fixture(fixture_name);
}

#[when("the fixture is compiled")]
fn when_fixture_is_compiled(world: &mut ContainerWorld) {
    world.compile_selected_fixture();
}

#[then(regex = r#"^compilation should fail with "(.*)"$"#)]
fn then_compilation_fails_with(world: &mut ContainerWorld, expected_error: String) {
    let outcome = world.compile_outcome();

    assert!(
        !outcome.succeeded,
        "Expected fixture '{}' to fail compilation, but it succeeded",
        outcome.fixture_name
    );
    assert!(
        outcome.stderr.contains(&expected_error),
        "Expected stderr for fixture '{}' to contain:\n  {expected_error}\nActual stderr:\n{}",
        outcome.fixture_name,
        outcome.stderr
    );
}

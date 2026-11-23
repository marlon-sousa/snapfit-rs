use compile_outcomes::{Diagnostic, Location, Outcome};
use cucumber::{given, then, when};

use crate::world::ContainerWorld;

#[given(expr = "an invalid container fixture {string}")]
fn given_invalid_container_fixture(world: &mut ContainerWorld, fixture_name: String) {
    world.select_fixture(fixture_name);
}

#[given("a newer snapfit library from another build sits beside the one the suite links to")]
fn given_stale_snapfit_library(world: &mut ContainerWorld) {
    world.plant_stale_library();
}

#[when("the fixture is compiled")]
fn when_fixture_is_compiled(world: &mut ContainerWorld) {
    world.compile_selected_fixture();
}

#[then(regex = r#"^compilation should fail with "(.*)"$"#)]
fn then_compilation_fails_with(world: &mut ContainerWorld, expected_error: String) {
    let (fixture, outcome) = world.compiled_fixture();
    let reported = outcome
        .errors()
        .iter()
        .any(|error| error.message.contains(&expected_error));

    assert!(
        outcome.is_failed(),
        "Expected fixture '{fixture}' to fail compilation, but it compiled. Its warnings:\n{}",
        rendered(outcome)
    );
    assert!(
        reported,
        "Expected an error for fixture '{fixture}' to contain:\n  {expected_error}\nThe compiler said:\n{}",
        rendered(outcome)
    );
}

#[then(expr = "the compiler output should not mention {string}")]
fn then_compiler_output_does_not_mention(world: &mut ContainerWorld, unwanted: String) {
    let (fixture, outcome) = world.compiled_fixture();
    let said = rendered(outcome);
    assert!(
        !said.contains(&unwanted),
        "Expected the compiler not to mention '{unwanted}' for fixture '{fixture}'. The compiler said:\n{said}"
    );
}

#[then(expr = "the error should point at line {int} of the fixture")]
fn then_error_points_at_line(world: &mut ContainerWorld, line: usize) {
    let (fixture, outcome) = world.compiled_fixture();
    let file_name = format!("{fixture}.rs");
    let pointed = outcome
        .errors()
        .iter()
        .any(|error| points_at(error, &file_name, line));
    assert!(
        pointed,
        "Expected an error for fixture '{fixture}' to point at line {line} of {file_name}. The compiler said:\n{}",
        rendered(outcome)
    );
}

#[then(expr = "the compiler should report {int} errors")]
fn then_compiler_reports_errors(world: &mut ContainerWorld, count: usize) {
    let (fixture, outcome) = world.compiled_fixture();
    assert_eq!(
        outcome.errors().len(),
        count,
        "Expected the compiler to report {count} errors for fixture '{fixture}'. The compiler said:\n{}",
        rendered(outcome)
    );
}

fn points_at(diagnostic: &Diagnostic, file_name: &str, line: usize) -> bool {
    diagnostic
        .location
        .as_ref()
        .is_some_and(|location| is_at(location, file_name, line))
}

fn is_at(location: &Location, file_name: &str, line: usize) -> bool {
    location.line == line
        && location
            .file
            .file_name()
            .is_some_and(|name| name == file_name)
}

/// Every error and warning, as a terminal shows them.
fn rendered(outcome: &Outcome) -> String {
    outcome
        .errors()
        .iter()
        .chain(outcome.warnings())
        .map(|diagnostic| diagnostic.rendered.as_str())
        .collect()
}

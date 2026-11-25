use cucumber::{given, then};

use crate::dispatched_containers::accounts::{Accounts, Ledgers, Spare};
use crate::dispatched_containers::checked::CheckedAccounts;
use crate::dispatched_containers::chosen::{self, Backend};
use crate::dispatched_containers::conversions::{Conversions, Convert};
use crate::dispatched_containers::ledgers::{Fake, Ledger};
use crate::dispatched_containers::written_by_a_macro::{Passed, PassedAccounts};
use crate::world::ContainerWorld;

#[given("a container whose fields hold a statically dispatched ledger")]
fn given_accounts(world: &mut ContainerWorld) {
    world.built.dispatch.accounts = Some(Accounts::new());
}

#[then(expr = "its {word} ledger should answer from {string}")]
fn then_field_answers_from(world: &mut ContainerWorld, field: String, expected: String) {
    let accounts = world.built.dispatch.accounts();
    let source = match field.as_str() {
        "main" => accounts.get_main().source(),
        "backup" => accounts.get_backup().source(),
        "cache" => accounts.get_cache().source(),
        "dynamic" => accounts.get_dynamic().source(),
        other => panic!("the Accounts container has no ledger field '{other}'"),
    };
    assert_eq!(source, expected);
}

#[then(expr = "its books should read from {string}")]
fn then_books_read_from(world: &mut ContainerWorld, expected: String) {
    assert_eq!(
        world.built.dispatch.accounts().get_books().source(),
        expected
    );
}

#[then(regex = r#"^a ledger built from (extra|the extension) should answer from "(.*)"$"#)]
fn then_built_ledger_answers(_world: &mut ContainerWorld, built: String, expected: String) {
    let ledger = match built.as_str() {
        "extra" => Ledgers::of(Fake),
        "the extension" => Ledgers::of(Spare),
        other => panic!("no ledger is built from '{other}'"),
    };
    assert_eq!(ledger.source(), expected);
}

#[then(expr = "building the checked accounts should fail with {string}")]
fn then_checked_fails(_world: &mut ContainerWorld, expected: String) {
    let error = CheckedAccounts::try_new()
        .err()
        .expect("the remote ledger has no host, so building it must fail");
    assert_eq!(error.to_string(), expected);
}

#[then(
    expr = "a container a macro_rules! writes should answer from {string}, and its extra from {string}"
)]
fn then_passed_answers(_world: &mut ContainerWorld, main: String, extra: String) {
    assert_eq!(PassedAccounts::new().get_main().source(), main);
    assert_eq!(Passed::of(Fake).source(), extra);
}

#[given("a container whose converters a trait with two type arguments dispatches")]
fn given_conversions(world: &mut ContainerWorld) {
    world.built.dispatch.conversions = Some(Conversions::new());
}

#[then(expr = "its {word} converter should turn {int} into {int}")]
fn then_converter_turns(world: &mut ContainerWorld, field: String, value: u8, expected: u16) {
    let conversions = world.built.dispatch.conversions();
    let converted = match field.as_str() {
        "shared" => conversions.get_shared().convert(value),
        "owned" => conversions.get_owned().convert(value),
        other => panic!("the Conversions container has no converter field '{other}'"),
    };
    assert_eq!(converted, expected);
}

#[given(expr = "a container whose factories choose its ledgers from the setting {backend}")]
fn given_chosen(world: &mut ContainerWorld, backend: Backend) {
    world.built.dispatch.chosen = Some(chosen::start_with(backend));
}

#[then(expr = "its shared and its owned ledger should both answer from {string}")]
fn then_chosen_answer_from(world: &mut ContainerWorld, expected: String) {
    let chosen = world.built.dispatch.chosen();
    assert_eq!(chosen.get_shared().source(), expected);
    assert_eq!(chosen.get_owned().source(), expected);
}

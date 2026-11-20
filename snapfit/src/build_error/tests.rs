use std::error::Error;

use crate::BuildError;

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("port {0} is taken")]
struct PortTaken(u16);

fn failed_with(source: impl Into<Box<dyn Error + Send + Sync>>) -> BuildError {
    BuildError {
        container: "AppContainer",
        field: "db",
        source: source.into(),
    }
}

#[test]
fn the_message_names_the_field_and_the_container_as_a_sentence() {
    let error = failed_with(PortTaken(5432));

    assert_eq!(
        error.to_string(),
        "snapfit could not build the field `db` of `AppContainer`"
    );
}

#[test]
fn the_container_and_the_field_are_read_by_their_accessors() {
    let error = failed_with(PortTaken(5432));

    assert_eq!(error.container(), "AppContainer");
    assert_eq!(error.field(), "db");
}

#[test]
fn the_field_s_own_error_is_kept_whole_as_the_source() {
    let error = failed_with(PortTaken(5432));

    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<PortTaken>());

    assert_eq!(source, Some(&PortTaken(5432)));
}

#[test]
fn a_build_error_is_send_sync_and_static() {
    let boxed: Box<dyn Error + Send + Sync + 'static> = failed_with("no config").into();

    assert_eq!(
        boxed.to_string(),
        "snapfit could not build the field `db` of `AppContainer`"
    );
}

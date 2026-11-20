use std::error::Error;
use std::io;

use super::BuildResult;
use crate::BuildError;

#[test]
fn a_success_is_the_field_s_value() {
    let built = Ok::<_, io::Error>(8080).into_build_result("AppContainer", "port");

    assert_eq!(built.ok(), Some(8080));
}

#[test]
fn a_failure_names_the_container_and_the_field_it_happened_in() {
    let failed =
        Err::<u16, _>(io::Error::other("no port")).into_build_result("AppContainer", "port");

    let error = failed.err();

    assert_eq!(
        error.as_ref().map(|error| error.container()),
        Some("AppContainer")
    );
    assert_eq!(error.as_ref().map(|error| error.field()), Some("port"));
}

/// The source's message once `error` failed a field.
fn source_of<E: Into<Box<dyn Error + Send + Sync>>>(error: E) -> Option<String> {
    Err::<(), E>(error)
        .into_build_result("AppContainer", "disk")
        .err()
        .and_then(|error| error.source().map(ToString::to_string))
}

#[test]
fn every_error_that_converts_into_a_boxed_error_is_kept_as_the_source() {
    let cases = [
        ("an error type", source_of(io::Error::other("disk full"))),
        ("a String", source_of(String::from("disk full"))),
        ("a str", source_of("disk full")),
        (
            "a boxed error",
            source_of(Box::<dyn Error + Send + Sync>::from("disk full")),
        ),
    ];

    for (kind, source) in cases {
        assert_eq!(source.as_deref(), Some("disk full"), "{kind}");
    }
}

#[test]
fn a_build_error_can_be_the_source_of_another() {
    let inner = BuildError {
        container: "Inner",
        field: "db",
        source: "no config".into(),
    };

    let source = Err::<(), _>(inner)
        .into_build_result("Outer", "inner")
        .err()
        .and_then(|error| error.source().map(ToString::to_string));

    assert_eq!(
        source.as_deref(),
        Some("snapfit could not build the field `db` of `Inner`")
    );
}

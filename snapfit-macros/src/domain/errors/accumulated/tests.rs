use super::*;
use crate::test_support::{cycle, name};

#[test]
fn no_errors_is_a_success() {
    assert_eq!(DomainErrors::check(Vec::new()), Ok(()));
}

#[test]
fn collecting_keeps_every_error_of_every_result_in_order() {
    let results = vec![
        Err(DomainErrors::from(cycle("a", &[]))),
        Ok(1),
        Err(DomainErrors::from(cycle("b", &[]))),
    ];

    let errors = DomainErrors::collect(results).unwrap_err();

    let subjects: Vec<_> = errors.iter().filter_map(DomainError::subject).collect();
    assert_eq!(subjects, [&name("a"), &name("b")]);
}

#[test]
fn collecting_successes_keeps_every_value() {
    let results: Vec<Result<i32, DomainErrors>> = vec![Ok(1), Ok(2)];

    assert_eq!(DomainErrors::collect(results), Ok(vec![1, 2]));
}

#[test]
fn both_keeps_the_errors_of_both_sides_in_order() {
    let first: Result<(), _> = Err(DomainErrors::from(cycle("a", &[])));
    let second: Result<(), _> = Err(DomainErrors::from(cycle("b", &[])));

    let errors = DomainErrors::both(first, second).unwrap_err();

    let subjects: Vec<_> = errors.iter().filter_map(DomainError::subject).collect();
    assert_eq!(subjects, [&name("a"), &name("b")]);
}

#[test]
fn both_gives_both_values_when_neither_failed() {
    let pair = DomainErrors::both(Ok::<_, DomainErrors>(1), Ok::<_, DomainErrors>("two"));

    assert_eq!(pair, Ok((1, "two")));
}

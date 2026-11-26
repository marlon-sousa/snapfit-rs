use quote::quote;

use super::*;
use crate::adapters::inbound::darling_models::ContainerAttrMeta;

fn nested(tokens: proc_macro2::TokenStream) -> Vec<NestedMeta> {
    NestedMeta::parse_meta_list(tokens).unwrap()
}

fn names(kept: &[NestedMeta]) -> Vec<String> {
    kept.iter().filter_map(name).collect()
}

fn copy_problems(copy: proc_macro2::TokenStream) -> Vec<ArgumentProblem> {
    let copy = nested(copy).remove(0);
    read_copy::<ContainerAttrMeta>(&copy).0
}

#[test]
fn settings_written_once_are_all_kept() {
    let (kept, repeated) = without_repeats(&nested(quote!(crate = di, mode = "sync")));
    assert_eq!(names(&kept), ["crate", "mode"]);
    assert!(repeated.is_empty());
}

#[test]
fn every_setting_after_the_first_of_its_name_is_taken_out() {
    let (kept, repeated) = without_repeats(&nested(quote!(
        crate = di,
        mode = "sync",
        crate,
        crate = 3,
        mode = "async"
    )));
    assert_eq!(names(&kept), ["crate", "mode"]);
    assert_eq!(names(&repeated), ["crate", "crate", "mode"]);
}

#[test]
fn a_literal_has_no_name_and_is_left_for_darling() {
    let (kept, repeated) = without_repeats(&nested(quote!("a", "a")));
    assert_eq!(kept.len(), 2);
    assert!(repeated.is_empty());
}

#[test]
fn a_setting_is_named_as_darling_spells_its_path() {
    let (_, repeated) = without_repeats(&nested(quote!(a::b, a::b)));
    assert_eq!(names(&repeated), ["a::b"]);
}

#[test]
fn a_copy_that_reads_is_refused_as_written_again() {
    assert_eq!(
        copy_problems(quote!(mode = "async")),
        [ArgumentProblem::RepeatedSetting("mode".to_owned())]
    );
}

#[test]
fn a_copy_whose_value_does_not_read_is_refused_for_its_value_too() {
    let problems = copy_problems(quote!(mode = 3));
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert_eq!(
        problems[0],
        ArgumentProblem::RepeatedSetting("mode".to_owned())
    );
    assert!(matches!(problems[1], ArgumentProblem::Unreadable(_)));
}

#[test]
fn a_copy_of_a_setting_the_attribute_does_not_know_is_left_to_the_first() {
    assert_eq!(copy_problems(quote!(bogus)), []);
}

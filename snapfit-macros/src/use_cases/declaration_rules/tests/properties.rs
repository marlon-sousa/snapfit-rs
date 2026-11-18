//! Tests: for any `deps` list, each field passed again to the same setter is
//! refused once, and each setter passed more than one field is refused once.

use std::collections::{HashMap, HashSet};

use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;

use super::*;

/// An entry of `deps`: a field and its label, each by position in `WORDS`.
type Entry = (usize, Option<usize>);

const WORDS: [&str; 4] = ["config", "db", "disk", "cache"];
const MOST_ENTRIES: usize = 8;

fn word(position: usize) -> String {
    WORDS[position].to_string()
}

fn entries() -> impl Strategy<Value = Vec<Entry>> {
    let position = 0..WORDS.len();
    vec((position.clone(), option::of(position)), 0..=MOST_ENTRIES)
}

fn prerequisite((field, label): &Entry) -> Prerequisite {
    match label {
        Some(label) => labelled(&word(*field), &word(*label)),
        None => Prerequisite::new(DependencyName::new(word(*field)).unwrap()),
    }
}

fn writer(entries: &[Entry], built_by_builder: bool) -> DependencyDeclaration<()> {
    let mut writer = dependency("writer", &[]);
    writer.prerequisites = entries.iter().map(prerequisite).collect();
    writer.construction = built_by_builder.then(builder);
    writer
}

/// An entry's field and setter, its label or else the field's own name.
fn field_and_setter((field, label): &Entry) -> (usize, usize) {
    (*field, label.unwrap_or(*field))
}

/// The field of each entry whose field and setter came before, once each,
/// in the order of its second coming, found apart from the rule.
fn fields_passed_again(entries: &[Entry]) -> Vec<String> {
    let start = (HashMap::new(), Vec::new());
    let (_, again) = entries.iter().map(field_and_setter).fold(start, with_use);
    again
}

/// How many times each field and setter came so far, and the fields that
/// came again.
type Uses = (HashMap<(usize, usize), usize>, Vec<String>);

fn with_use((mut times, mut again): Uses, (field, setter): (usize, usize)) -> Uses {
    let count = times.entry((field, setter)).or_default();
    *count += 1;
    if *count == 2 {
        again.push(word(field));
    }
    (times, again)
}

/// Each setter a second field is passed to, once, in the order that field
/// comes, found apart from the rule.
fn setters_used_again(entries: &[Entry]) -> Vec<String> {
    let start = (HashMap::new(), Vec::new());
    let (_, again) = entries
        .iter()
        .map(field_and_setter)
        .fold(start, with_setter);
    again
}

/// The fields each setter was passed so far, and the setters passed a
/// second field.
type Setters = (HashMap<usize, HashSet<usize>>, Vec<String>);

fn with_setter((mut fields_of, mut again): Setters, (field, setter): (usize, usize)) -> Setters {
    let fields = fields_of.entry(setter).or_default();
    if fields.insert(field) && fields.len() == 2 {
        again.push(word(setter));
    }
    (fields_of, again)
}

fn repeated_fields(errors: &[DomainError]) -> Vec<String> {
    errors.iter().filter_map(repeated_field).collect()
}

fn repeated_field(error: &DomainError) -> Option<String> {
    match error {
        DomainError::RepeatedPrerequisite { prerequisite, .. } => Some(prerequisite.to_string()),
        _ => None,
    }
}

fn setters_refused(errors: &[DomainError]) -> Vec<String> {
    errors.iter().filter_map(setter_refused).collect()
}

fn setter_refused(error: &DomainError) -> Option<String> {
    match error {
        DomainError::DuplicateSetter { setter, .. } => Some(setter.to_string()),
        _ => None,
    }
}

proptest! {
    #[test]
    fn each_field_passed_again_to_the_same_setter_is_refused_once(
        entries in entries(),
        built_by_builder in any::<bool>(),
    ) {
        let errors = rule_violations(&container(vec![writer(&entries, built_by_builder)]));

        prop_assert_eq!(repeated_fields(&errors), fields_passed_again(&entries));
    }

    #[test]
    fn a_builder_refuses_each_setter_passed_more_than_one_field_once(entries in entries()) {
        let errors = rule_violations(&container(vec![writer(&entries, true)]));
        let refusals = fields_passed_again(&entries).len() + setters_used_again(&entries).len();

        prop_assert_eq!(setters_refused(&errors), setters_used_again(&entries));
        prop_assert_eq!(errors.len(), refusals);
    }

    #[test]
    fn without_a_builder_each_label_is_refused_and_no_setter_is(entries in entries()) {
        let errors = rule_violations(&container(vec![writer(&entries, false)]));
        let labels = entries.iter().filter(|(_, label)| label.is_some()).count();

        prop_assert!(setters_refused(&errors).is_empty());
        prop_assert_eq!(errors.len(), fields_passed_again(&entries).len() + labels);
    }
}

//! Error: the sentences several errors build from lists of names.

use crate::domain::names::DependencyName;

/// "a depends on b, which depends on c, which depends on a", or "a depends
/// on itself", so that no arrow has to be read aloud.
pub(super) fn cycle_sentence(start: &DependencyName, through: &[DependencyName]) -> String {
    let Some((first, rest)) = through.split_first() else {
        return format!("{start} depends on itself");
    };
    rest.iter()
        .chain([start])
        .fold(format!("{start} depends on {first}"), add_link)
}

fn add_link(sentence: String, name: &DependencyName) -> String {
    format!("{sentence}, which depends on {name}")
}

pub(super) fn join_names(names: &[DependencyName]) -> String {
    names
        .iter()
        .map(DependencyName::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

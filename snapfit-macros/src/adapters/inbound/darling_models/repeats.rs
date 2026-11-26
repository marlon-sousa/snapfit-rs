//! Adapter: the settings of an attribute written more than once, taken out
//! before darling reads the rest, so that each is refused in snapfit's
//! words. Plays the role of its parent.

use std::collections::HashSet;

use darling::FromMeta;
use darling::ast::NestedMeta;
use darling::util::path_to_string;

use super::unreadable;
use crate::domain::ArgumentProblem;

/// The settings with the first of each name, in the order written, and
/// every setting that repeats a name before it.
pub(super) fn without_repeats(nested: &[NestedMeta]) -> (Vec<NestedMeta>, Vec<NestedMeta>) {
    let mut seen = HashSet::new();
    nested
        .iter()
        .cloned()
        .partition(|setting| is_first_of_its_name(&mut seen, setting))
}

/// The refusals of a copy written again, read alone, and its settings if it
/// reads: it is refused as written again, beside whatever is wrong with its
/// value. A name the attribute does not know is not refused again: darling's
/// problem with the first one already names it.
pub(super) fn read_copy<Settings: FromMeta>(
    copy: &NestedMeta,
) -> (Vec<ArgumentProblem>, Option<Settings>) {
    let name = name(copy);
    match Settings::from_list(std::slice::from_ref(copy)) {
        Ok(settings) => (repeated(name).collect(), Some(settings)),
        Err(err) if name.as_deref().is_some_and(|name| is_unknown(name, &err)) => {
            (Vec::new(), None)
        }
        Err(err) => (repeated(name).chain([unreadable(err)]).collect(), None),
    }
}

fn repeated(name: Option<String>) -> impl Iterator<Item = ArgumentProblem> {
    name.map(ArgumentProblem::RepeatedSetting).into_iter()
}

fn is_first_of_its_name(seen: &mut HashSet<String>, setting: &NestedMeta) -> bool {
    name(setting).is_none_or(|name| seen.insert(name))
}

/// Whether darling refused the setting for its name, in the words its own
/// `unknown_field` writes, which only its private kind otherwise tells.
fn is_unknown(name: &str, err: &darling::Error) -> bool {
    err.to_string()
        .starts_with(&darling::Error::unknown_field(name).to_string())
}

/// The name a setting is written with, such as `crate` in `crate = di`, as
/// darling spells it; a literal has none.
fn name(setting: &NestedMeta) -> Option<String> {
    match setting {
        NestedMeta::Meta(meta) => Some(path_to_string(meta.path())),
        NestedMeta::Lit(_) => None,
    }
}

#[cfg(test)]
mod tests;

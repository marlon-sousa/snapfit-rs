use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::sorted_files;

const PROBLEMS_HEADING: &str = "## What goes wrong";
const ANSWERS_HEADING: &str = "## Which problem this solves";

/// Every problem Part 1 names is answered by a feature chapter, and every
/// answer names a problem that Part 1 states.
#[test]
fn every_named_problem_is_answered_and_every_answer_names_one() {
    let chapters_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("chapters");
    let pages: Vec<String> = sorted_files(&chapters_dir, "md")
        .iter()
        .map(|path| fs::read_to_string(path).expect("failed to read a chapter"))
        .collect();

    let named = bold_names_under(&pages, PROBLEMS_HEADING);
    let answered = bold_names_under(&pages, ANSWERS_HEADING);

    assert!(
        !named.is_empty(),
        "no chapter names a problem under \"{PROBLEMS_HEADING}\""
    );
    let unanswered: Vec<&String> = named.difference(&answered).collect();
    let unknown: Vec<&String> = answered.difference(&named).collect();
    assert!(
        unanswered.is_empty(),
        "these problems are never answered under \"{ANSWERS_HEADING}\": {unanswered:?}"
    );
    assert!(
        unknown.is_empty(),
        "these answers name no problem of Part 1: {unknown:?}"
    );
}

/// The bold names that open the numbered items of every section with this heading.
fn bold_names_under(pages: &[String], heading: &str) -> BTreeSet<String> {
    pages
        .iter()
        .flat_map(|page| {
            page.lines()
                .skip_while(move |line| *line != heading)
                .skip(1)
                .take_while(|line| !line.starts_with("## "))
        })
        .filter_map(opening_bold_name)
        .collect()
}

/// `1. **Nothing is shared.** ...` and `1. **Nothing is shared** (...)` both name `Nothing is shared`.
fn opening_bold_name(line: &str) -> Option<String> {
    let (number, rest) = line.split_once(". **")?;
    let (name, _) = rest.split_once("**")?;
    number
        .chars()
        .all(|ch| ch.is_ascii_digit())
        .then(|| name.trim_end_matches('.').to_string())
}

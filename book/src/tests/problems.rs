use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::{section_lines, sorted_files};

const PROBLEMS_HEADING: &str = "## What we found";
const ANSWERS_HEADING: &str = "## Which problem this solves";

#[test]
fn every_named_problem_is_answered_and_every_answer_names_one() {
    let pages = read_chapters();
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

#[test]
fn the_problem_table_lists_every_named_problem() {
    let pages = read_chapters();
    let named = bold_names_under(&pages, PROBLEMS_HEADING);
    let tabled: BTreeSet<String> = pages
        .iter()
        .flat_map(|page| table_rows(page, ANSWERS_HEADING))
        .filter_map(first_cell)
        .collect();
    assert_eq!(
        tabled, named,
        "the table under \"{ANSWERS_HEADING}\" must list exactly the problems Part 1 names"
    );
}

fn read_chapters() -> Vec<String> {
    let chapters_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("chapters");
    sorted_files(&chapters_dir, "md")
        .into_iter()
        .map(read_chapter)
        .collect()
}

fn read_chapter(path: PathBuf) -> String {
    fs::read_to_string(path).expect("failed to read a chapter")
}

/// The bold names that open the numbered items of every section with this heading.
fn bold_names_under(pages: &[String], heading: &str) -> BTreeSet<String> {
    pages
        .iter()
        .flat_map(|page| section_lines(page, heading))
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
        .then_some(name.trim_end_matches('.').to_string())
}

/// The body rows of the first table in the section with this heading.
fn table_rows<'a>(page: &'a str, heading: &str) -> impl Iterator<Item = &'a str> {
    let mut lines = section_lines(page, heading);
    lines.find(|line| line.starts_with("| :---"));
    lines.take_while(|line| line.starts_with('|'))
}

/// `| Nothing is shared | ... |` names `Nothing is shared`.
fn first_cell(row: &str) -> Option<String> {
    let cell = row.strip_prefix('|')?.split('|').next()?.trim();
    (!cell.is_empty()).then_some(cell.to_string())
}

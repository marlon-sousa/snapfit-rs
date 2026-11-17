use std::fs;
use std::path::Path;

use super::sorted_files;

/// Words that only an unstable or planned API of docs/SPEC.md §0 uses.
const UNSTABLE_OR_PLANNED: &[&str] = &[
    "unstable",
    "transient",
    "builder",
    "Builders",
    "async",
    ".await",
    "init()",
    "mode =",
    "dispatch",
    "fork_with",
    "parent =",
    "parent(",
    "introspection",
    "dependency_graph",
    "#[external",
    "before_init",
];

/// The folders of the edition, each with the kind of file it holds.
const EDITION: &[(&str, &str)] = &[
    ("chapters", "md"),
    ("examples", "rs"),
    ("expected", "txt"),
    ("compile-errors", "rs"),
    ("compile-errors", "txt"),
];

#[test]
fn the_edition_never_shows_an_unstable_or_planned_api() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let found: Vec<String> = EDITION
        .iter()
        .flat_map(|(dir, ext)| sorted_files(&book_dir.join(dir), ext))
        .flat_map(|path| {
            let content = fs::read_to_string(&path).expect("failed to read a book file");
            let shown = path
                .strip_prefix(book_dir)
                .unwrap_or(&path)
                .display()
                .to_string();
            UNSTABLE_OR_PLANNED
                .iter()
                .filter(|word| content.contains(*word))
                .map(|word| format!("book/{shown} contains \"{word}\""))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        found.is_empty(),
        "The edition teaches only the stable API, but:\n{}",
        found.join("\n")
    );
}

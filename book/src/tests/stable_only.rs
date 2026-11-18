use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use super::sorted_files;

/// Words that only an unstable or planned API of docs/SPEC.md §0 uses.
const UNSTABLE_OR_PLANNED: &[&str] = &[
    "unstable",
    "transient",
    "builder =",
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

/// A folder of the edition and the extension of the files it holds.
struct EditionFiles {
    dir: &'static str,
    extension: &'static str,
}

const EDITION: &[EditionFiles] = &[
    EditionFiles {
        dir: "chapters",
        extension: "md",
    },
    EditionFiles {
        dir: "examples",
        extension: "rs",
    },
    EditionFiles {
        dir: "expected",
        extension: "txt",
    },
    EditionFiles {
        dir: "compile-errors",
        extension: "rs",
    },
    EditionFiles {
        dir: "compile-errors",
        extension: "txt",
    },
];

#[test]
fn the_edition_never_shows_an_unstable_or_planned_api() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let found: Vec<String> = EDITION
        .iter()
        .flat_map(|files| files_of(book_dir, files))
        .flat_map(|path| unstable_words_in(book_dir, &path))
        .collect();
    assert!(
        found.is_empty(),
        "The edition teaches only the stable API, but:\n{}",
        found.join("\n")
    );
}

fn files_of(book_dir: &Path, files: &EditionFiles) -> Vec<PathBuf> {
    sorted_files(&book_dir.join(files.dir), files.extension)
}

/// One line per unstable or planned word the file contains.
fn unstable_words_in(book_dir: &Path, path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).expect("failed to read a book file");
    let shown = path.strip_prefix(book_dir).unwrap_or(path).display();
    UNSTABLE_OR_PLANNED
        .iter()
        .filter(|word| content.contains(*word))
        .map(|word| found_in(&shown, word))
        .collect()
}

fn found_in(path: impl Display, word: &str) -> String {
    format!("book/{path} contains \"{word}\"")
}

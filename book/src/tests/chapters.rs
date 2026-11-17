use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::{section_lines, sorted_files};

/// The level 2 sections every feature chapter ends with, in this order.
const FEATURE_CLOSING_SECTIONS: &[&str] = &["## Which problem this solves", "## Remember"];

/// The level 2 section every problem chapter of Part 1 ends with.
const PROBLEM_CLOSING_SECTIONS: &[&str] = &["## What we found"];

const PROBLEM_PART_HEADING: &str = "## Part 1:";

#[test]
fn chapters_table_of_contents_and_structure_are_valid() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let chapters_dir = book_dir.join("chapters");
    let readme = fs::read_to_string(chapters_dir.join("README.md"))
        .expect("missing book/chapters/README.md");

    let linked_files = extract_linked_chapter_files(readme.lines());
    let problem_files = extract_linked_chapter_files(section_lines(&readme, PROBLEM_PART_HEADING));
    assert!(
        !problem_files.is_empty(),
        "book/chapters/README.md must list the problem chapters under a heading starting with \"{PROBLEM_PART_HEADING}\""
    );
    let actual_files = actual_chapter_filenames(&chapters_dir);
    assert_eq!(
        linked_files, actual_files,
        "book/chapters/README.md links must match the exact set of chapter files in book/chapters/"
    );

    for file_name in &actual_files {
        let content = fs::read_to_string(chapters_dir.join(file_name)).unwrap();
        verify_single_h1(file_name, &content);
        if is_structured_chapter(file_name) {
            let closing = if problem_files.contains(file_name) {
                PROBLEM_CLOSING_SECTIONS
            } else {
                FEATURE_CLOSING_SECTIONS
            };
            verify_closing_sections(file_name, &content, closing);
        }
        verify_mermaid_accessibility(file_name, &content);
    }

    verify_all_examples_and_compile_errors_are_used(book_dir, &chapters_dir);
}

fn extract_linked_chapter_files<'a>(lines: impl Iterator<Item = &'a str>) -> HashSet<String> {
    lines
        .filter_map(|line| {
            let (_, rest) = line.split_once("](")?;
            let (target, _) = rest.split_once(')')?;
            (target.ends_with(".md") && !target.contains('/')).then(|| target.to_string())
        })
        .collect()
}

fn actual_chapter_filenames(chapters_dir: &Path) -> HashSet<String> {
    sorted_files(chapters_dir, "md")
        .into_iter()
        .filter_map(|p| p.file_name()?.to_str().map(str::to_string))
        .filter(|name| name != "README.md")
        .collect()
}

fn is_structured_chapter(file_name: &str) -> bool {
    file_name == "quick-start.md"
        || file_name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit())
}

fn verify_single_h1(file_name: &str, content: &str) {
    let h1_count = non_fenced_lines(content)
        .into_iter()
        .filter(|line| line.starts_with("# "))
        .count();
    assert_eq!(
        h1_count, 1,
        "{file_name} must have exactly one level-1 heading, found {h1_count}"
    );
}

/// The chapter's own sections come first, then the closing ones, each once and in order.
fn verify_closing_sections(file_name: &str, content: &str, closing: &[&str]) {
    let h2_headings: Vec<&str> = non_fenced_lines(content)
        .into_iter()
        .filter(|line| line.starts_with("## "))
        .collect();
    let own_count = h2_headings.len().saturating_sub(closing.len());
    let (own, last) = h2_headings.split_at(own_count);
    assert_eq!(
        last, closing,
        "{file_name} must end with the ## sections {closing:?}, in this order"
    );
    assert!(
        !own.is_empty() && own.iter().all(|heading| !closing.contains(heading)),
        "{file_name} must have its own ## sections before {closing:?}, and use those only at the end"
    );
}

fn non_fenced_lines(content: &str) -> Vec<&str> {
    let mut in_fence = false;
    let mut result = Vec::new();
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            result.push(line);
        }
    }
    result
}

fn verify_mermaid_accessibility(file_name: &str, content: &str) {
    for part in content.split("```mermaid").skip(1) {
        let block = part.split("```").next().unwrap_or("");
        assert!(
            block.contains("accTitle:") && block.contains("accDescr:"),
            "{file_name}: every mermaid diagram must declare accTitle and accDescr"
        );
    }
}

fn verify_all_examples_and_compile_errors_are_used(book_dir: &Path, chapters_dir: &Path) {
    let all_chapters_text: String = sorted_files(chapters_dir, "md")
        .into_iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    for dir_name in ["examples", "compile-errors"] {
        for rs_file in sorted_files(&book_dir.join(dir_name), "rs") {
            let stem = rs_file.file_stem().and_then(|s| s.to_str()).unwrap();
            let needle = format!("name: {stem}");
            assert!(
                all_chapters_text.contains(&needle),
                "book/{dir_name}/{stem}.rs is never referenced by any chapter directive"
            );
        }
    }
}

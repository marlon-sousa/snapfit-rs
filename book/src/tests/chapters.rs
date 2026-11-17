use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::{section_lines, sorted_files};

const FEATURE_SECTIONS: &[&str] = &[
    "## What you want to do",
    "## The code",
    "## How it works",
    "## Which problem this solves",
    "## Remember",
];

const PROBLEM_SECTIONS: &[&str] = &[
    "## What you want to do",
    "## The code",
    "## What goes wrong",
    "## Remember",
];

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
            let sections = if problem_files.contains(file_name) {
                PROBLEM_SECTIONS
            } else {
                FEATURE_SECTIONS
            };
            verify_sections(file_name, &content, sections);
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

fn verify_sections(file_name: &str, content: &str, sections: &[&str]) {
    let h2_headings: Vec<&str> = non_fenced_lines(content)
        .into_iter()
        .filter(|line| line.starts_with("## "))
        .collect();
    assert_eq!(
        h2_headings, sections,
        "{file_name} must have the required ## sections in exact order"
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

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_COMPILER_TOKENS: &[&str] = &[
    "syn::",
    "use syn",
    "quote::",
    "quote!",
    "use quote",
    "darling::",
    "use darling",
    "proc_macro::",
    "use proc_macro",
    "proc_macro2::",
    "use proc_macro2",
];

const SOFT_LINE_LIMIT: usize = 200;
const ISOLATED_DIRS: &[&str] = &["snapfit-macros/src/domain", "snapfit-macros/src/use_cases"];
const WORKSPACE_CRATES: &[&str] = &["snapfit", "snapfit-macros", "xtask", "tests", "book"];
const CRATE_SUBDIRS: &[&str] = &["src", "examples"];

pub(crate) fn check_architectural_invariants(root: &Path) -> bool {
    println!("\n>>> Checking Architectural Invariants & File Lengths...");
    let mut violations = collect_all_compiler_leaks(root);
    violations.extend(collect_all_layout_and_size_violations(root));

    if !violations.is_empty() {
        eprintln!("❌ Architectural Boundary Check Failed:");
        violations.iter().for_each(|v| eprintln!("{v}"));
        return false;
    }

    println!(
        "✅ Architectural Invariants: 0 compiler leaks, no mod.rs, all file lengths compliant."
    );
    true
}

fn collect_all_compiler_leaks(root: &Path) -> Vec<String> {
    ISOLATED_DIRS
        .iter()
        .map(|rel| root.join(rel))
        .filter(|dir| dir.exists())
        .flat_map(|dir| collect_rs_files(&dir))
        .flat_map(|path| scan_file_for_leaks(root, &path))
        .collect()
}

fn scan_file_for_leaks(root: &Path, path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let rel = relative_display(root, path);
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .flat_map(|(idx, line)| find_line_leaks(&rel, idx + 1, line))
        .collect()
}

fn find_line_leaks(rel: &str, line_no: usize, line: &str) -> Vec<String> {
    FORBIDDEN_COMPILER_TOKENS
        .iter()
        .filter(|token| line.contains(**token))
        .map(|token| format!("  [LEAK] {rel}:{line_no}: forbidden compiler token '{token}'"))
        .collect()
}

fn collect_all_layout_and_size_violations(root: &Path) -> Vec<String> {
    WORKSPACE_CRATES
        .iter()
        .flat_map(|crate_name| {
            CRATE_SUBDIRS
                .iter()
                .map(move |sub| root.join(crate_name).join(sub))
        })
        .filter(|dir| dir.exists())
        .flat_map(|dir| collect_rs_files(&dir))
        .flat_map(|path| check_file_layout_and_size(root, &path))
        .collect()
}

fn check_file_layout_and_size(root: &Path, path: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let rel = relative_display(root, path);

    if path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
        issues.push(format!(
            "  [MOD.RS] {rel}: forbidden 'mod.rs' detected. Use modern Rust module layout."
        ));
    }
    if let Ok(content) = fs::read_to_string(path) {
        let count = content.lines().count();
        if count > SOFT_LINE_LIMIT {
            issues.push(format!(
                "  [SIZE] {rel}: {count} lines exceeds ~{SOFT_LINE_LIMIT} line soft limit"
            ));
        }
    }
    issues
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .flat_map(expand_dir_or_rs_file)
        .collect();
    files.sort();
    files
}

fn expand_dir_or_rs_file(path: PathBuf) -> Vec<PathBuf> {
    if path.is_dir() {
        collect_rs_files(&path)
    } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
        vec![path]
    } else {
        Vec::new()
    }
}

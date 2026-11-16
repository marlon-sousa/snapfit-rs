#![deny(warnings)]

mod arch_check;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const VALID_MODES: &[&str] = &[
    "all",
    "ci",
    "fmt",
    "fmt-check",
    "clippy",
    "arch",
    "arch-check",
    "test",
    "doc",
    "book",
    "book-check",
];

fn main() -> ExitCode {
    let mode = env::args().nth(1).unwrap_or_else(|| "all".to_string());
    if !VALID_MODES.contains(&mode.as_str()) {
        eprintln!(
            "❌ Unknown mode '{mode}'. Available modes: {}",
            VALID_MODES.join(", ")
        );
        return ExitCode::FAILURE;
    }

    if execute_mode(&repo_root(), &mode) {
        println!("\n🎉 Quality Gate PASSED: All checks clean with zero errors!");
        ExitCode::SUCCESS
    } else {
        eprintln!("\n❌ Quality Gate FAILED: See details above.");
        ExitCode::FAILURE
    }
}

fn execute_mode(root: &Path, mode: &str) -> bool {
    let run_all = matches!(mode, "all" | "ci");
    let mut ok = true;

    if run_all || matches!(mode, "fmt" | "fmt-check") {
        ok = run_cargo(root, &["fmt", "--all", "--", "--check"], false) && ok;
    }
    if run_all || mode == "clippy" {
        ok = run_cargo(
            root,
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            false,
        ) && ok;
    }
    if run_all || matches!(mode, "arch" | "arch-check") {
        ok = arch_check::check_architectural_invariants(root) && ok;
    }
    if run_all || mode == "test" {
        ok = run_cargo(root, &["test", "--workspace", "--all-targets"], false) && ok;
    }
    if run_all || mode == "doc" {
        ok = run_cargo(root, &["doc", "--workspace", "--no-deps"], true) && ok;
    }
    if matches!(mode, "book" | "book-check") {
        ok = run_cargo(
            root,
            &["test", "-p", "snapfit_book", "--all-targets"],
            false,
        ) && ok;
    }
    ok
}

fn repo_root() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir)
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn run_cargo(root: &Path, args: &[&str], deny_rustdoc_warnings: bool) -> bool {
    println!("\n>>> Running: cargo {}", args.join(" "));
    let mut cmd = Command::new("cargo");
    cmd.args(args).current_dir(root);
    if deny_rustdoc_warnings {
        cmd.env("RUSTDOCFLAGS", "-D warnings");
    }
    cmd.status().is_ok_and(|s| s.success())
}

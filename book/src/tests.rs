mod chapters;
mod compile_errors;
mod directives;
mod problems;
mod readme;
mod stable_only;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn examples_compile_run_and_match_expected_output() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let examples_dir = book_dir.join("examples");
    let expected_dir = book_dir.join("expected");
    let bless = env::var("SNAPFIT_BOOK_BLESS").is_ok();

    build_all_example_binaries(book_dir);

    for rs_file in sorted_files(&examples_dir, "rs") {
        let name = file_stem(&rs_file);
        let actual = run_example(&name);
        let expected_path = expected_dir.join(format!("{name}.txt"));

        if bless {
            fs::write(&expected_path, &actual).expect("failed to write expected output");
            continue;
        }

        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("missing book/expected/{name}.txt"))
            .replace("\r\n", "\n");
        assert_eq!(
            actual, expected,
            "stdout mismatch for book example '{name}'"
        );
    }
}

fn build_all_example_binaries(book_dir: &Path) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args(["build", "-p", "snapfit_book", "--examples", "--quiet"])
        .current_dir(book_dir)
        .status()
        .expect("failed to invoke cargo build --examples for snapfit_book");
    assert!(
        status.success(),
        "cargo build -p snapfit_book --examples failed"
    );
}

fn run_example(name: &str) -> String {
    let bin = direct_example_binary(name);
    let output = Command::new(&bin).output().unwrap_or_else(|e| {
        panic!(
            "failed to execute example '{name}' at {}: {e}",
            bin.display()
        )
    });
    assert!(
        output.status.success(),
        "book example '{name}' failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn direct_example_binary(name: &str) -> PathBuf {
    let examples_out = target_deps_dir()
        .parent()
        .expect("missing debug dir")
        .join("examples");
    let suffix = env::consts::EXE_SUFFIX;
    let direct = examples_out.join(format!("{name}{suffix}"));
    assert!(
        direct.exists(),
        "compiled example binary '{}' does not exist at {}",
        name,
        direct.display()
    );
    direct
}

pub(super) fn sorted_files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .expect("failed to read directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some(ext))
        .collect();
    files.sort();
    files
}

pub(super) fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .expect("valid UTF-8 stem")
        .to_string()
}

pub(super) fn target_deps_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .expect("failed to locate target/debug/deps directory")
}

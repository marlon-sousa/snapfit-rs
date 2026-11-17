use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use compile_outcomes::{Compilation, Dependency, Edition, Outcome, Outcomes, compile};

use super::{file_stem, sorted_files, target_deps_dir};

/// Every fixture, compiled once against the stable snapfit the book teaches, in a work folder beside the book's own build.
static OUTCOMES: LazyLock<Outcomes> = LazyLock::new(|| {
    let work_dir = target_deps_dir()
        .parent()
        .expect("missing debug dir")
        .join("book_compile_errors");
    let compilation = Compilation::new(Edition::E2024, work_dir)
        .snippets_in(compile_errors_dir())
        .dependency(Dependency::path("snapfit", book_dir().join("../snapfit")))
        .lockfile(book_dir().join("../Cargo.lock"));
    compile(&compilation).unwrap_or_else(|errors| {
        panic!("the book's compile-error fixtures could not be compiled:\n{errors}")
    })
});

#[test]
fn compile_error_fixtures_fail_and_match_recorded_diagnostics() {
    let bless = env::var("SNAPFIT_BOOK_BLESS").is_ok();

    for rs_file in sorted_files(&compile_errors_dir(), "rs") {
        let name = file_stem(&rs_file);
        let actual = recorded_form(&rs_file, &name);
        let txt_path = recorded_path(&name);

        if bless {
            fs::write(&txt_path, &actual).expect("failed to write recorded compile error");
            continue;
        }

        assert_eq!(
            actual,
            recorded(&txt_path, &name),
            "compiler diagnostic mismatch for book/compile-errors/{name}.rs"
        );
    }
}

#[test]
fn a_newer_snapfit_library_from_another_build_changes_no_recorded_diagnostic() {
    let name = "ch11_dependency_cycle";
    let _stale = StaleLibrary::plant();

    let actual = recorded_form(&compile_errors_dir().join(format!("{name}.rs")), name);

    assert_eq!(actual, recorded(&recorded_path(name), name));
}

/// A file named like a snapfit library, the newest in the book's dependency folder, removed when dropped.
struct StaleLibrary(PathBuf);

impl StaleLibrary {
    fn plant() -> Self {
        let path = target_deps_dir().join("libsnapfit-0000000000000000.rlib");
        fs::write(&path, "not a library").expect("failed to write the stale snapfit library");
        Self(path)
    }
}

impl Drop for StaleLibrary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn book_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn compile_errors_dir() -> PathBuf {
    book_dir().join("compile-errors")
}

fn recorded_path(name: &str) -> PathBuf {
    compile_errors_dir().join(format!("{name}.txt"))
}

fn recorded(txt_path: &Path, name: &str) -> String {
    fs::read_to_string(txt_path)
        .unwrap_or_else(|_| panic!("missing book/compile-errors/{name}.txt"))
        .replace("\r\n", "\n")
}

/// The fixture's errors as the book records them: rendered as a terminal shows them, with the fixture's path written alike on every platform.
fn recorded_form(rs_file: &Path, name: &str) -> String {
    let outcome = OUTCOMES
        .get(name)
        .unwrap_or_else(|| panic!("no outcome for book/compile-errors/{name}.rs"));
    assert!(
        outcome.is_failed(),
        "book compile-error fixture '{name}' unexpectedly compiled cleanly"
    );
    normalize(&rendered_errors(outcome), rs_file, name)
}

/// Every error as a terminal shows it, in the order rustc wrote them.
fn rendered_errors(outcome: &Outcome) -> String {
    outcome
        .errors()
        .iter()
        .map(|error| error.rendered.as_str())
        .collect()
}

fn normalize(rendered: &str, rs_file: &Path, name: &str) -> String {
    let rel_display = format!("book/compile-errors/{name}.rs");
    let raw_path = rs_file.to_string_lossy();
    let slash_path = raw_path.replace('\\', "/");
    let normalized = rendered
        .replace("\r\n", "\n")
        .replace(raw_path.as_ref(), &rel_display)
        .replace(&slash_path, &rel_display);

    let filtered: Vec<&str> = normalized
        .lines()
        .filter(|line| !line.contains("this error originates in the attribute macro"))
        .collect();
    format!("{}\n", filtered.join("\n").trim())
}

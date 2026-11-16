use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use super::{file_stem, sorted_files, target_deps_dir};

#[test]
fn compile_error_fixtures_fail_and_match_recorded_diagnostics() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ce_dir = book_dir.join("compile-errors");
    let deps_dir = target_deps_dir();
    let snapfit_rlib = find_latest_snapfit_rlib(&deps_dir);
    let out_dir = deps_dir.join("book_compile_errors_out");
    fs::create_dir_all(&out_dir).expect("failed to create book_compile_errors_out");
    let bless = env::var("SNAPFIT_BOOK_BLESS").is_ok();

    for rs_file in sorted_files(&ce_dir, "rs") {
        let name = file_stem(&rs_file);
        let actual = compile_failing_fixture(&rs_file, &name, &snapfit_rlib, &deps_dir, &out_dir);
        let txt_path = ce_dir.join(format!("{name}.txt"));

        if bless {
            fs::write(&txt_path, &actual).expect("failed to write recorded compile error");
            continue;
        }

        let expected = fs::read_to_string(&txt_path)
            .unwrap_or_else(|_| panic!("missing book/compile-errors/{name}.txt"))
            .replace("\r\n", "\n");
        assert_eq!(
            actual, expected,
            "compiler diagnostic mismatch for book/compile-errors/{name}.rs"
        );
    }
}

fn compile_failing_fixture(
    rs_file: &Path,
    name: &str,
    snapfit_rlib: &Path,
    deps_dir: &Path,
    out_dir: &Path,
) -> String {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc)
        .arg("--crate-type=lib")
        .arg("--edition=2024")
        .arg("--color=never")
        .arg("-Awarnings")
        .arg("--crate-name")
        .arg(name)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(format!("--extern=snapfit={}", snapfit_rlib.display()))
        .arg(format!("-Ldependency={}", deps_dir.display()))
        .arg(rs_file)
        .output()
        .expect("failed to invoke rustc on book compile-error fixture");

    assert!(
        !output.status.success(),
        "book compile-error fixture '{name}' unexpectedly compiled cleanly"
    );
    normalize_stderr(&String::from_utf8_lossy(&output.stderr), rs_file, name)
}

fn normalize_stderr(stderr: &str, rs_file: &Path, name: &str) -> String {
    let rel_display = format!("book/compile-errors/{name}.rs");
    let raw_path = rs_file.to_string_lossy();
    let slash_path = raw_path.replace('\\', "/");
    let normalized = stderr
        .replace("\r\n", "\n")
        .replace(raw_path.as_ref(), &rel_display)
        .replace(&slash_path, &rel_display);

    let filtered: Vec<&str> = normalized
        .lines()
        .filter(|line| {
            !line.starts_with("error: aborting due to ")
                && !line.starts_with("For more information about this error")
                && !line.contains("this error originates in the attribute macro")
        })
        .collect();
    format!("{}\n", filtered.join("\n").trim())
}

fn find_latest_snapfit_rlib(deps_dir: &Path) -> PathBuf {
    fs::read_dir(deps_dir)
        .expect("failed to read deps dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            (fname == "libsnapfit.rlib" || fname.starts_with("libsnapfit-"))
                && fname.ends_with(".rlib")
        })
        .max_by_key(|p| modified_time(p))
        .expect("failed to find compiled libsnapfit.rlib")
}

fn modified_time(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

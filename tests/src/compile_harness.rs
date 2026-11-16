use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::SystemTime;

use rayon::prelude::*;

static FIXTURE_CACHE: OnceLock<HashMap<String, CompileOutcome>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileOutcome {
    pub fixture_name: String,
    pub succeeded: bool,
    pub stderr: String,
}

struct CompilerContext {
    rustc: String,
    deps_dir: PathBuf,
    wireup_rlib: PathBuf,
    wireup_mtime: SystemTime,
    out_dir: PathBuf,
}

pub fn compile_fixture(fixture_name: &str) -> CompileOutcome {
    FIXTURE_CACHE
        .get_or_init(compile_all_fixtures_in_parallel)
        .get(fixture_name)
        .cloned()
        .unwrap_or_else(|| panic!("unknown compile-fail fixture '{fixture_name}'"))
}

fn compile_all_fixtures_in_parallel() -> HashMap<String, CompileOutcome> {
    let ctx = CompilerContext::discover();
    discover_fixture_files(&fixtures_dir())
        .into_par_iter()
        .map(|path| resolve_or_compile_fixture(&path, &ctx))
        .map(|outcome| (outcome.fixture_name.clone(), outcome))
        .collect()
}

fn resolve_or_compile_fixture(path: &Path, ctx: &CompilerContext) -> CompileOutcome {
    let name = fixture_stem(path);
    if let Some(cached_stderr) = read_valid_cached_stderr(path, &name, ctx) {
        return CompileOutcome {
            fixture_name: name,
            succeeded: false,
            stderr: cached_stderr,
        };
    }
    compile_and_cache_fixture(path, name, ctx)
}

fn read_valid_cached_stderr(
    fixture_path: &Path,
    name: &str,
    ctx: &CompilerContext,
) -> Option<String> {
    let cache_file = stderr_cache_path(&ctx.out_dir, name);
    let cache_mtime = modified_timestamp(&cache_file);
    let fixture_mtime = modified_timestamp(fixture_path);
    let is_fresh = cache_mtime > fixture_mtime && cache_mtime > ctx.wireup_mtime;

    is_fresh
        .then(|| fs::read_to_string(cache_file).ok())
        .flatten()
}

fn compile_and_cache_fixture(
    path: &Path,
    fixture_name: String,
    ctx: &CompilerContext,
) -> CompileOutcome {
    let output = Command::new(&ctx.rustc)
        .arg("--crate-type=lib")
        .arg("--edition=2024")
        .arg("--color=never")
        .arg("--crate-name")
        .arg(&fixture_name)
        .arg("--out-dir")
        .arg(&ctx.out_dir)
        .arg(format!("--extern=wireup={}", ctx.wireup_rlib.display()))
        .arg(format!("-Ldependency={}", ctx.deps_dir.display()))
        .arg(path)
        .output()
        .expect("failed to invoke rustc on compile-fail fixture");

    let succeeded = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !succeeded {
        let _ = fs::write(stderr_cache_path(&ctx.out_dir, &fixture_name), &stderr);
    }

    CompileOutcome {
        fixture_name,
        succeeded,
        stderr,
    }
}

fn stderr_cache_path(out_dir: &Path, name: &str) -> PathBuf {
    out_dir.join(format!("{name}.stderr"))
}

impl CompilerContext {
    fn discover() -> Self {
        let deps_dir = current_deps_dir();
        let wireup_rlib = find_latest_wireup_rlib(&deps_dir);
        let wireup_mtime = modified_timestamp(&wireup_rlib);
        let out_dir = prepare_out_dir(&deps_dir);
        let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());

        Self {
            rustc,
            deps_dir,
            wireup_rlib,
            wireup_mtime,
            out_dir,
        }
    }
}

fn current_deps_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .expect("failed to locate target/debug/deps directory from test executable")
}

fn find_latest_wireup_rlib(deps_dir: &Path) -> PathBuf {
    fs::read_dir(deps_dir)
        .expect("failed to read target deps directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_wireup_rlib(path))
        .max_by_key(|path| modified_timestamp(path))
        .expect("failed to find compiled libwireup.rlib in target deps directory")
}

fn is_wireup_rlib(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    (file_name == "libwireup.rlib" || file_name.starts_with("libwireup-"))
        && file_name.ends_with(".rlib")
}

fn modified_timestamp(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn prepare_out_dir(deps_dir: &Path) -> PathBuf {
    let out_dir = deps_dir.join("compile_fail_out");
    fs::create_dir_all(&out_dir).expect("failed to create compile_fail_out directory");
    out_dir
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/compile_fail")
}

fn discover_fixture_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("failed to read fixtures/compile_fail directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("rs"))
        .collect()
}

fn fixture_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .expect("fixture file must have a valid UTF-8 stem")
        .to_string()
}

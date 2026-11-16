#![deny(warnings)]

use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let Some(python) = find_python_binary() else {
        eprintln!("❌ Python is required to run xtask gates across Windows, Linux, and macOS.");
        return ExitCode::FAILURE;
    };

    run_gate_script(python)
}

fn find_python_binary() -> Option<&'static str> {
    ["python3", "python", "py"]
        .into_iter()
        .find(|candidate| Command::new(candidate).arg("--version").output().is_ok())
}

fn gate_script_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    std::path::Path::new(&manifest_dir).parent().map_or_else(
        || PathBuf::from("scripts/gate.py"),
        |p| p.join("scripts").join("gate.py"),
    )
}

fn run_gate_script(python: &str) -> ExitCode {
    let status = Command::new(python)
        .arg(gate_script_path())
        .args(env::args().skip(1))
        .status()
        .expect("failed to execute python gate runner");

    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

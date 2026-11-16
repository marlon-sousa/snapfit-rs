#![deny(warnings)]

use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // Prefer python3 or python or py launcher for cross-platform gate execution
    let python_candidates = ["python3", "python", "py"];
    let mut chosen_python = None;

    for candidate in python_candidates {
        if Command::new(candidate).arg("--version").output().is_ok() {
            chosen_python = Some(candidate);
            break;
        }
    }

    let Some(python) = chosen_python else {
        eprintln!("❌ Python is required to run xtask gates across Windows, Linux, and macOS.");
        std::process::exit(1);
    };

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let script_path = std::path::Path::new(&manifest_dir)
        .parent()
        .map(|p| p.join("scripts").join("gate.py"))
        .unwrap_or_else(|| "scripts/gate.py".into());

    let mut cmd = Command::new(python);
    cmd.arg(script_path);
    cmd.args(&args);

    let status = cmd.status().expect("failed to execute python gate runner");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

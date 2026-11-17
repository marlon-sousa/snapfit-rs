//! A file named like a snapfit library from another build, the newest beside the one this suite links to.

use std::fs;
use std::path::{Path, PathBuf};

/// Removed from the suite's dependency folder when dropped, so that no later build meets it.
pub struct StaleLibrary(PathBuf);

impl StaleLibrary {
    pub fn plant() -> Self {
        let path = suite_dependency_dir().join("libsnapfit-0000000000000000.rlib");
        fs::write(&path, "not a library").expect("failed to write the stale snapfit library");
        Self(path)
    }
}

impl Drop for StaleLibrary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// The folder Cargo built this suite's dependencies into, such as `target/debug/deps`, which holds the suite itself.
fn suite_dependency_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .expect("failed to locate the folder of the test executable")
}

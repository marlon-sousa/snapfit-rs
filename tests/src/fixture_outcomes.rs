//! Every compile-fail fixture, compiled once with compile-outcomes against this build of snapfit.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use compile_outcomes::{Compilation, Dependency, Edition, Outcome, Outcomes, compile};

/// The build the suite runs, which the fixtures compile against and which names their work folder.
const BUILD: &str = if cfg!(feature = "unstable") {
    "unstable"
} else {
    "stable"
};

static OUTCOMES: LazyLock<Outcomes> = LazyLock::new(|| {
    compile(&compilation()).unwrap_or_else(|errors| {
        panic!("the compile-fail fixtures could not be compiled:\n{errors}")
    })
});

pub fn fixture_outcome(fixture_name: &str) -> &'static Outcome {
    OUTCOMES
        .get(fixture_name)
        .unwrap_or_else(|| panic!("unknown compile-fail fixture '{fixture_name}'"))
}

fn compilation() -> Compilation {
    let work_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("compile_fail")
        .join(BUILD);
    Compilation::new(Edition::E2024, work_dir)
        .snippets_in(from_manifest_dir("fixtures/compile_fail"))
        .dependency(snapfit())
        .lockfile(from_manifest_dir("../Cargo.lock"))
}

fn snapfit() -> Dependency {
    let snapfit = Dependency::path("snapfit", from_manifest_dir("../snapfit"));
    if cfg!(feature = "unstable") {
        snapfit.features(["unstable"])
    } else {
        snapfit
    }
}

fn from_manifest_dir(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

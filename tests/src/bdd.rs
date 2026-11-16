#![deny(warnings)]

mod codegen_containers;
mod compile_harness;
mod containers;
mod services;
mod steps;
#[cfg(feature = "unstable")]
mod unstable_containers;
#[cfg(feature = "unstable")]
mod unstable_world;
mod world;

use std::path::Path;

use cucumber::World as _;
use futures::executor::block_on;
use world::ContainerWorld;

/// The stable suite runs without the `unstable` feature so it can prove the
/// gate; the unstable suite is a separate build (`cargo xtask test` runs both).
const SUITE: &str = if cfg!(feature = "unstable") {
    "features/unstable"
} else {
    "features/stable"
};

fn main() {
    let feature_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SUITE);

    block_on(
        ContainerWorld::cucumber()
            .fail_on_skipped()
            .run_and_exit(feature_path),
    );
}

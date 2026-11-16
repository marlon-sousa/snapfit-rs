#![deny(warnings)]

mod compile_harness;
mod containers;
mod services;
mod steps;
mod world;

use std::path::Path;

use cucumber::World as _;
use futures::executor::block_on;
use world::ContainerWorld;

fn main() {
    let feature_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("features/container_resolution.feature");

    block_on(
        ContainerWorld::cucumber()
            .fail_on_skipped()
            .run_and_exit(feature_path),
    );
}

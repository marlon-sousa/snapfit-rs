#!/usr/bin/env bash
# Prints the version Cargo.toml gives the snapfit package, the workspace's
# version, which every published crate shares.
set -euo pipefail
cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.name == "snapfit") | .version' \
    | tr -d '\r'

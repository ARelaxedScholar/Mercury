#!/usr/bin/env bash

set -euo pipefail

cargo fmt --all -- --check
cargo test --locked --workspace --all-targets
cargo test --locked --workspace --all-targets --all-features
cargo test --locked --workspace --doc --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo package --locked --allow-dirty -p orichalcum-definition
# Cargo cannot assemble dependent archives until their exact lockstep dependencies are
# present on crates.io. Listing still validates each package's selected file set; the
# workspace tests above compile the dependency chain from source.
cargo package --locked --allow-dirty -p orichalcum-macros --list >/dev/null
cargo package --locked --allow-dirty -p orichalcum --list >/dev/null

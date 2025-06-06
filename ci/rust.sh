#!/bin/sh
set -ex
cargo fmt --all --check --quiet
cargo clippy --workspace --all-targets --all-features --quiet -- -Dwarnings
cargo test --workspace --all-features --quiet -- --nocapture

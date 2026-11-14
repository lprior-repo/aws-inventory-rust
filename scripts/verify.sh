#!/usr/bin/env bash
set -euo pipefail

# Run from the repository root. The explicit catalog split avoids accidentally
# compiling all 437 service SDKs in an ordinary application validation job.
scripts/check-source-size.sh
scripts/check-function-size.sh
cargo fmt --all --check
cargo run -q -p catalog-tool -- parent-candidates --root . --check
# The model checks read the pinned submodule .local/aws-models.
scripts/check-models.sh
cargo run -q -p catalog-tool --locked -- models report --check >/dev/null
cargo run -q -p catalog-tool --locked -- models fields --check
cargo run -q -p catalog-tool --locked -- models endpoints --check >/dev/null
cargo run -q -p catalog-tool --locked -- classes --check >/dev/null
cargo run -p inventory-engine --example resource_views --locked -- --check
cargo check --workspace --exclude inventory-sdk-catalog --all-targets --all-features --locked -j1
cargo check -p inventory-sdk-catalog --no-default-features --locked
cargo clippy --workspace --exclude inventory-sdk-catalog --lib --bins --examples --all-features --locked -j1 -- -D warnings
cargo clippy -p inventory-sdk-catalog --example audit --locked -j1 -- -D warnings
cargo test --workspace --exclude inventory-sdk-catalog --all-features --locked -j1
# Keep the final binary on the same workspace/dev-feature graph as the tests
# and integration examples, avoiding a second SDK rebuild and binary replacement.
cargo build --workspace --exclude inventory-sdk-catalog --all-features --bins --examples --locked -j1

# Explicit opt-in checks, requiring their own environment/resources:
# cargo run -p inventory-engine --release --example benchmark --locked -- 10000
# FALKORDB_TEST_URL=redis://127.0.0.1:6379 cargo test -p inventory-engine --test graph live_falkordb_replay_and_parameter_round_trip -- --ignored --exact
# cargo check -p inventory-sdk-catalog --features all-services --locked -j1

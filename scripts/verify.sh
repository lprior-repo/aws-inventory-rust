#!/usr/bin/env bash
set -euo pipefail

# Run from the repository root. The explicit catalog split avoids accidentally
# compiling all 437 service SDKs in an ordinary application validation job.
# Install scripts/requirements.txt into the interpreter selected below first.
INVENTORY_PYTHON="${INVENTORY_PYTHON:-python3}"
scripts/check-source-size.sh
cargo fmt --all --check
"$INVENTORY_PYTHON" scripts/generate_sdk_catalog.py --check
"$INVENTORY_PYTHON" catalog/upstream/build_parent_catalog.py --check
"$INVENTORY_PYTHON" -m unittest discover -s catalog/upstream -p 'test_*.py'
"$INVENTORY_PYTHON" -m unittest discover -s scripts/catalog -p 'test_*.py'
"$INVENTORY_PYTHON" -m unittest discover -s scripts/emulator -p 'test_*.py'
"$INVENTORY_PYTHON" -m unittest discover -s scripts/mutation -p 'test_*.py'
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
# python scripts/generate_sdk_catalog.py --check --verify-cache
# cargo run -p inventory-engine --release --example benchmark --locked -- 10000
# FALKORDB_TEST_URL=redis://127.0.0.1:6379 cargo test -p inventory-engine --test graph live_falkordb_replay_and_parameter_round_trip -- --ignored --exact
# cargo check -p inventory-sdk-catalog --features all-services --locked -j1

#!/usr/bin/env bash
# Produces every regenerable catalog artifact that the build and tests read.
#
# Inputs: the pinned .local/aws-models submodule (models.lock) and the tracked
# catalog inputs, including the Git LFS objects. Outputs: the files listed in
# catalog/.gitignore, which are never committed. Generation follows the moon
# task graph (universal -> parent-candidates -> report -> fields -> endpoints ->
# classes -> plans -> overlay -> kinds -> compile), then the tracked registries are
# checked, protocol fixtures are fetched when their script exists, and
# MANIFEST.sha256 is written last so it records the freshly produced files.
#
# Bash 3.2 compatible (macOS /bin/bash): no arrays, mapfile or associative maps.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
    printf 'bootstrap-catalog: cargo not found; install the toolchain in rust-toolchain.toml\n' >&2
    exit 1
fi

# 1. Models submodule at the commit recorded in models.lock.
git submodule update --init --depth 1 -- .local/aws-models
scripts/check-models.sh

# 2. Git LFS objects for the non-regenerable inputs (pointers otherwise).
if git lfs version >/dev/null 2>&1; then
    git lfs pull
else
    mise x git-lfs@3.8.1 -- git lfs pull
fi

# 3. Compile the generators once, then run each stage.
cargo build -p inventory-harness --release --locked
cargo build -p catalog-tool --release --locked
harness() {
    cargo run -q --release -p inventory-harness --locked -- "$@"
}
catalog() {
    cargo run -q --release -p catalog-tool --locked -- "$@"
}

harness catalog universal --root .
catalog parent-candidates --root . --write
catalog models report --write
catalog models fields --check
catalog models endpoints --write
catalog classes --write
catalog plans --write
catalog overlay --source .local/aws-models --root . --write
catalog kinds generate --write
catalog coverage --write
catalog models compile --write
catalog iam-policy --write
catalog manifest --write

if [[ -f scripts/bootstrap-protocol-tests.sh ]]; then
    scripts/bootstrap-protocol-tests.sh
fi

printf 'catalog bootstrap complete\n'

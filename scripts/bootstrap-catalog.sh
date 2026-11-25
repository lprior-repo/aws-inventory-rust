#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
    printf 'bootstrap-catalog: cargo not found; install the toolchain in rust-toolchain.toml\n' >&2
    exit 1
fi

git submodule update --init --depth 1 -- .local/aws-models
scripts/check-models.sh

if git lfs version >/dev/null 2>&1; then
    git lfs pull
else
    mise x git-lfs@3.8.1 -- git lfs pull
fi

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

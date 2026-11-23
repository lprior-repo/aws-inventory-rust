#!/usr/bin/env bash
# Runs the baseline end-to-end flow against the pinned emulator: install the
# runtime if it is missing, build, then run the ignored sweep_e2e suite, which
# starts its own emulator and FalkorDB, seeds, sweeps, projects and verifies.
# Needs: rustup, a C toolchain, git, curl, gpg, dpkg-deb, bsdtar, make.
# The sweep feature loads AWS profiles without the legacy typed service adapters.
set -euo pipefail
cd "$(dirname "$0")/.."

runtime=.local/runtime
emulator="$runtime/unpacked/opt/aws-emulator/bin/aws-emulator"
graph="$runtime/graph"

for tool in cargo git curl gpg dpkg-deb bsdtar make; do
    if ! command -v "$tool" >/dev/null; then
        printf 'missing required tool: %s\n' "$tool" >&2
        exit 1
    fi
done

cargo build --locked -p inventory-harness -j1

if [[ ! -x "$emulator" ]]; then
    cargo run --locked -p inventory-harness -- install "$runtime"
fi
if [[ ! -d "$graph" ]]; then
    cargo run --locked -p inventory-harness -- install-graph "$graph"
fi

cargo test --locked --features sweep --test sweep_e2e -j1 -- --ignored --test-threads 1

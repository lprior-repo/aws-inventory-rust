#!/usr/bin/env bash
# Install missing pinned runtimes, build, then run the owned PostgreSQL e2e flow.
# Requires INVENTORY_POSTGRES_URL (or --postgres-url passed to this script).
# Whole-service selectors report all unsupported and failed operations in this
# catalog; they do not silently reduce EC2 to the archive's four-callable baseline.
set -euo pipefail
cd "$(dirname "$0")/.."

runtime=.local/runtime
emulator="${INVENTORY_EMULATOR_BINARY:-$runtime/unpacked/opt/aws-emulator/bin/aws-emulator}"
graph="$runtime/graph"

for tool in cargo git curl gpg dpkg-deb bsdtar make; do
    if ! command -v "$tool" >/dev/null; then
        printf 'missing required tool: %s\n' "$tool" >&2
        exit 1
    fi
done

cargo build --locked -p inventory-harness -p aws-inventory --features sweep -j1

if [[ ! -x "$emulator" ]]; then
    cargo run --locked -p inventory-harness -- install "$runtime"
fi
if [[ ! -f "${INVENTORY_FALKOR_MODULE:-$graph/falkordb.so}" ]]; then
    cargo run --locked -p inventory-harness -- install-graph "$graph"
fi

work=$(mktemp -d)
printf 'e2e work: %s\n' "$work"
exec cargo run --locked -p inventory-harness -- e2e run --work "$work" \
    --service ec2 "$@"

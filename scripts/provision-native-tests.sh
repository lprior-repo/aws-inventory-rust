#!/usr/bin/env bash
# Explicit Moon prerequisite, never invoked by ordinary cargo unit tests.
set -euo pipefail
umask 077
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
STATE="$ROOT/.local/native-tests"
cd "$ROOT"

if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
    printf 'Native integration provisioning requires Linux x86_64.\n' >&2
    exit 1
fi
for tool in cargo timeout curl git gpg gpgv bsdtar make cc ar; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf 'Missing native integration dependency: %s\n' "$tool" >&2
        exit 1
    fi
done
[[ -r /proc/self/fd/0 ]]
[[ -s "$ROOT/catalog/aws-api.catalog" ]]
mkdir -p "$STATE"

# Keep the executable path deterministic even when the caller uses a custom
# Cargo target directory/target. Moon holds cargo-workspace for this whole task.
unset CARGO_BUILD_TARGET
export CARGO_TARGET_DIR="$ROOT/target"
timeout --signal=TERM --kill-after=30s 60m cargo build \
    --package aws-inventory --package inventory-harness \
    --features aws-inventory/aws --bins --locked -j1
HARNESS="$CARGO_TARGET_DIR/debug/inventory-harness"

# Reuse the public installers: they own URLs, digests, signature verification,
# cache attestation and build receipts. No second pin manifest belongs in CI.
# Graph downloads already have a 128 MiB byte bound and a 180-second deadline.
timeout --signal=TERM --kill-after=30s 40m "$HARNESS" \
    install-graph "$STATE/graph" --jobs 2
(
    # The emulator installer's curl has a 300-second per-request deadline.
    # Bound every output file to 2 GiB as well, including package/clone files,
    # without changing its production download or trust policy.
    ulimit -f 2097152
    timeout --signal=TERM --kill-after=30s 20m "$HARNESS" \
        install "$STATE/emulator" --root "$ROOT"
)

# Missing inputs remain hard failures; native tests create their own isolated
# emulator endpoints, credentials, CA and data. No shared service is started.
[[ -s "$STATE/graph/redis-8.10.3.tar.gz" ]]
[[ -s "$STATE/graph/falkordb.so" ]]
[[ -x "$STATE/emulator/unpacked/opt/aws-emulator/bin/aws-emulator" ]]
[[ -x "$CARGO_TARGET_DIR/debug/aws-inventory" ]]

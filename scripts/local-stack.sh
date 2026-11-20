#!/usr/bin/env bash
# Local, authenticated AWS fixtures and FalkorDB. Never uses a live AWS profile.
set -euo pipefail
umask 077
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
STATE="$ROOT/.local"
mkdir -p "$STATE"
RUNTIME=$(realpath -m "$STATE/runtime")
EMULATOR="$RUNTIME/emulator/unpacked/opt/aws-emulator/bin/aws-emulator"
REDIS="$RUNTIME/graph/redis-8.10.3/src/redis-server"
REDIS_CLI="$RUNTIME/graph/redis-8.10.3/src/redis-cli"
MODULE="$RUNTIME/graph/falkordb.so"
BINARY="$ROOT/target/debug/aws-inventory"
IDENTITY=$(printf '%s' "$ROOT" | sha256sum | cut -c1-12)
UNIT="aws-inventory-local-$IDENTITY"
cd "$ROOT"

require_install() {
    for executable in "$EMULATOR" "$REDIS" "$REDIS_CLI" "$BINARY" \
        "$ROOT/target/debug/examples/emulator_branches"; do
        if [[ ! -x "$executable" ]]; then
            printf 'Missing %s. Run scripts/local-stack.sh install first.\n' "$executable" >&2
            exit 1
        fi
    done
}

clear_aws() {
    local name
    while IFS= read -r name; do unset "$name"; done < <(compgen -v AWS_)
    unset FALKORDB_URL FALKORDB_TEST_URL INVENTORY_EMULATOR_TEST
}

fixture_command() {
    local destination=$1
    shift
    require_install
    unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
    exec inventory-harness mutate \
        --emulator "$EMULATOR" --binary "$BINARY" \
        --branch-probe "$ROOT/target/debug/examples/emulator_branches" \
        --output "$destination" --redis-server "$REDIS" --falkor-module "$MODULE" \
        --command-deadline 604800 -- "$@"
}

case "${1:-help}" in
install)
    mkdir -p "$RUNTIME"
    inventory-harness install "$RUNTIME/emulator"
    inventory-harness install-graph "$RUNTIME/graph" --jobs 2
    cargo build --locked --workspace --exclude inventory-sdk-catalog --all-features --bins --examples -j1
    ;;
sdk-build)
    exec systemd-run --user --scope --collect --unit="$UNIT-sdk-build" \
        -p MemoryHigh=24G -p MemoryMax=32G -p MemorySwapMax=0 \
        cargo build -p inventory-sdk-catalog --features all-services --locked -j2 \
        --target-dir "$STATE/all-sdk-target"
    ;;
start)
    require_install
    if systemctl --user is-active --quiet "$UNIT"; then
        printf 'Local stack already running: %s\n' "$UNIT"
        exit 0
    fi
    systemd-run --user --collect --unit "$UNIT" \
        --property=WorkingDirectory="$ROOT" --property=KillMode=mixed \
        --property=TimeoutStopSec=90 --property=UMask=0077 --property=SuccessExitStatus=143 \
        --property=MemoryHigh=20G --property=MemoryMax=32G --setenv=PATH="$PATH" \
        --setenv=CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" \
        --setenv=RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
        -- "$ROOT/scripts/local-stack.sh" serve
    printf 'Starting and verifying fixtures. Run scripts/local-stack.sh status for readiness.\n'
    ;;
serve)
    unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
    destination="$STATE/sessions/$(date -u +%Y%m%dT%H%M%S)-$$"
    mkdir -p "$STATE/sessions"
    fixture_command "$destination" "$ROOT/scripts/local-stack.sh" ready "$destination"
    ;;
ready)
    # The verified harness supplies path-based emulator credentials and TLS trust.
    # Persist paths only; the generated keys remain in the emulator credential file.
    destination=${2:?session directory required}
    temporary=$(mktemp "$STATE/session.env.XXXXXX")
    for name in AWS_ENDPOINT_URL AWS_SHARED_CREDENTIALS_FILE AWS_CONFIG_FILE \
        AWS_CA_BUNDLE SSL_CERT_FILE AWS_REGION AWS_DEFAULT_REGION \
        AWS_EC2_METADATA_DISABLED AWS_DISABLE_HOST_PREFIX_INJECTION NO_PROXY; do
        printf 'export %s=%q\n' "$name" "${!name}" >> "$temporary"
    done
    printf 'export FALKORDB_URL=%q\n' "$FALKORDB_TEST_URL" >> "$temporary"
    printf 'export INVENTORY_LOCAL_SESSION=%q\n' "$destination" >> "$temporary"
    mv "$temporary" "$STATE/session.env"
    printf 'READY: authenticated local emulator and FalkorDB; session %s\n' "$destination"
    trap 'exit 0' TERM INT
    while :; do sleep 30 & wait "$!" || true; done
    ;;
status)
    systemctl --user show "$UNIT" --property=ActiveState --property=SubState --property=MainPID
    if systemctl --user is-active --quiet "$UNIT" && [[ -f "$STATE/session.env" ]]; then
        source "$STATE/session.env"
        active_pid=$(systemctl --user show "$UNIT" --property=MainPID --value)
        if [[ "$INVENTORY_LOCAL_SESSION" == "$STATE/sessions/"*-"$active_pid" && -r "$SSL_CERT_FILE" ]] \
            && curl --fail --silent --max-time 5 --cacert "$SSL_CERT_FILE" "$AWS_ENDPOINT_URL/__admin/health" >/dev/null \
            && [[ $("$REDIS_CLI" -u "$FALKORDB_URL" ping) == PONG ]]; then
            printf 'READY\nEmulator: %s\nFalkorDB: %s\nSession: %s\n' \
                "$AWS_ENDPOINT_URL" "$FALKORDB_URL" "$INVENTORY_LOCAL_SESSION"
            exit 0
        fi
    fi
    printf 'Not ready. Inspect: journalctl --user -u %s -n 30\n' "$UNIT" >&2
    exit 1
    ;;
stop)
    systemctl --user stop "$UNIT"
    printf 'Stopped local stack. Saved inventory and evidence are retained.\n'
    ;;
cli)
    shift
    require_install
    "$0" status >/dev/null
    clear_aws
    source "$STATE/session.env"
    exec "$BINARY" "$@"
    ;;
verify)
    require_install
    destination="$STATE/verification/$(date -u +%Y%m%dT%H%M%S)-$$"
    mkdir -p "$STATE/verification"
    inventory-harness verify-delivery \
        --binary "$BINARY" --redis-cli "$REDIS_CLI" \
        --database "$destination/inventory.sqlite" \
        --output "$destination/delivery-verification.json"
    ;;
logs)
    exec journalctl --user -u "$UNIT" -n 50 --no-pager
    ;;
help|-h|--help)
    printf '%s\n' 'Usage: scripts/local-stack.sh {install|sdk-build|start|status|stop|cli ARGS...|verify|logs}' \
        'start: supervised, seeded loopback AWS emulator + FalkorDB; no sudo or live AWS.' \
        'cli: invoke the collector using only the verified local emulator environment.' \
        'verify: fresh fixtures, all Rust tests, exact SQLite/export/graph checks and replay.' \
        'sdk-build: compile all 437 SDKs in a separate target with a 32 GiB memory cap.' \
        'Services stop after seven days or on reboot; use start again to recreate fixtures.'
    ;;
*) printf 'Unknown command: %s\n' "$1" >&2; exit 2 ;;
esac

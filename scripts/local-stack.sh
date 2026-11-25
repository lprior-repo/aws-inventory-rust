#!/usr/bin/env bash
set -euo pipefail
umask 077
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
STATE_DIR="${AWS_INVENTORY_STATE_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/aws-inventory}"
RUNTIME="$STATE_DIR/runtime"
EMULATOR="$RUNTIME/emulator/unpacked/opt/aws-emulator/bin/aws-emulator"
REDIS_CLI="$RUNTIME/graph/redis-8.10.3/src/redis-cli"
REDIS_SERVER="${FALKORDB_REDIS_SERVER:-$RUNTIME/graph/redis-8.10.3/src/redis-server}"
MODULE="${FALKORDB_MODULE:-$RUNTIME/graph/falkordb.so}"
BINARY="$ROOT/target/debug/aws-inventory"
HARNESS="$ROOT/target/debug/inventory-harness"
PID_FILE="$STATE_DIR/serve.pid"
LOG_FILE="$STATE_DIR/serve.log"
SESSION_ENV="$STATE_DIR/session.env"
FALKOR_STATE="$STATE_DIR/falkordb.env"
FALKORDB_IMAGE="docker.io/falkordb/falkordb:6.0.4"
FALKORDB_PORT="${FALKORDB_PORT:-16379}"
cd "$ROOT"

require_install() {
    for executable in "$EMULATOR" "$REDIS_CLI" "$BINARY" "$HARNESS" \
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
    "$HARNESS" mutate \
        --emulator "$EMULATOR" --binary "$BINARY" \
        --branch-probe "$ROOT/target/debug/examples/emulator_branches" \
        --output "$destination" --redis-server "$REDIS_SERVER" --falkor-module "$MODULE" \
        --command-deadline 604800 -- "$@"
}

serve_pid() {
    [[ -f "$PID_FILE" ]] || return 1
    cat "$PID_FILE"
}

is_running() {
    local pid
    pid=$(serve_pid 2>/dev/null) || return 1
    [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

redis_ping() {
    local hostport=${1#redis://}
    (
        exec 3<>"/dev/tcp/${hostport%:*}/${hostport##*:}" || exit 1
        printf 'PING\r\n' >&3
        reply=""
        IFS= read -r -t 5 reply <&3 || true
        [[ $reply == '+PONG'* ]]
    ) 2>/dev/null
}

wait_for_falkordb() {
    local attempt
    for attempt in $(seq 1 60); do
        redis_ping "$1" && return 0
        sleep 1
    done
    printf 'FalkorDB did not answer PING at %s. See %s\n' "$1" "$LOG_FILE" >&2
    return 1
}

container_engine() {
    local engine
    for engine in docker podman; do
        if command -v "$engine" >/dev/null 2>&1 && "$engine" info >/dev/null 2>&1; then
            printf '%s\n' "$engine"
            return 0
        fi
    done
    return 1
}

record_falkordb() {
    local temporary
    mkdir -p "$STATE_DIR"
    temporary=$(mktemp "$STATE_DIR/falkordb.env.XXXXXX")
    printf 'FALKORDB_ENGINE=%q\nFALKORDB_NAME=%q\nFALKORDB_PID=%q\nFALKORDB_TEST_URL=%q\n' \
        "$1" "$2" "$3" "$4" >"$temporary"
    mv "$temporary" "$FALKOR_STATE"
}

start_falkordb() {
    local engine name pid url="redis://127.0.0.1:$FALKORDB_PORT"
    if [[ -n "${FALKORDB_TEST_URL:-}" ]]; then
        record_falkordb external "" "" "$FALKORDB_TEST_URL"
        wait_for_falkordb "$FALKORDB_TEST_URL"
    elif engine=$(container_engine); then
        name="aws-inventory-falkordb-$FALKORDB_PORT"
        "$engine" rm -f "$name" >/dev/null 2>&1 || true
        "$engine" run -d --name "$name" -p "127.0.0.1:$FALKORDB_PORT:6379" \
            "$FALKORDB_IMAGE" >>"$LOG_FILE" 2>&1
        record_falkordb "$engine" "$name" "" "$url"
        wait_for_falkordb "$url"
    else
        if [[ ! -x "$REDIS_SERVER" || ! -f "$MODULE" ]]; then
            printf 'No docker or podman, and no redis-server (%s) with FalkorDB module (%s).\n' \
                "$REDIS_SERVER" "$MODULE" >&2
            printf 'Install a container engine, or run scripts/local-stack.sh install.\n' >&2
            return 1
        fi
        mkdir -p "$STATE_DIR/falkordb-data"
        "$REDIS_SERVER" --bind 127.0.0.1 --port "$FALKORDB_PORT" \
            --loadmodule "$MODULE" --save "" --appendonly no \
            --dir "$STATE_DIR/falkordb-data" >>"$LOG_FILE" 2>&1 &
        pid=$!
        record_falkordb redis "" "$pid" "$url"
        wait_for_falkordb "$url"
    fi
}

stop_falkordb() {
    [[ -f "$FALKOR_STATE" ]] || return 0
    local FALKORDB_ENGINE="" FALKORDB_NAME="" FALKORDB_PID="" FALKORDB_TEST_URL=""
    source "$FALKOR_STATE"
    case $FALKORDB_ENGINE in
        docker | podman) "$FALKORDB_ENGINE" rm -f "$FALKORDB_NAME" >/dev/null 2>&1 || true ;;
        redis) [[ -n $FALKORDB_PID ]] && kill "$FALKORDB_PID" 2>/dev/null || true ;;
    esac
    rm -f "$FALKOR_STATE"
}

case "${1:-help}" in
install)
    mkdir -p "$RUNTIME"
    cargo build --locked -p inventory-harness --all-features -j1
    "$HARNESS" install "$RUNTIME/emulator"
    "$HARNESS" install-graph "$RUNTIME/graph" --jobs 2
    cargo build --locked --workspace --exclude inventory-sdk-catalog --all-features --bins --examples -j1
    ;;
sdk-build)
    cargo build -p inventory-sdk-catalog --features all-services --locked -j2 \
        --target-dir "$STATE_DIR/all-sdk-target"
    ;;
start)
    require_install
    if is_running; then
        printf 'Local stack already running (pid %s).\n' "$(serve_pid)"
        exit 0
    fi
    mkdir -p "$STATE_DIR"
    set -m
    nohup "$ROOT/scripts/local-stack.sh" serve >>"$LOG_FILE" 2>&1 &
    printf '%s\n' "$!" >"$PID_FILE"
    set +m
    printf 'Starting and verifying fixtures. Run scripts/local-stack.sh status for readiness.\n'
    ;;
serve)
    destination="$STATE_DIR/sessions/$(date -u +%Y%m%dT%H%M%S)-$$"
    mkdir -p "$STATE_DIR/sessions"
    start_falkordb
    fixture_command "$destination" "$ROOT/scripts/local-stack.sh" ready "$destination"
    ;;
ready)
    destination=${2:?session directory required}
    FALKORDB_TEST_URL=""
    source "$FALKOR_STATE"
    temporary=$(mktemp "$STATE_DIR/session.env.XXXXXX")
    for name in AWS_ENDPOINT_URL AWS_SHARED_CREDENTIALS_FILE AWS_CONFIG_FILE \
        AWS_CA_BUNDLE SSL_CERT_FILE AWS_REGION AWS_DEFAULT_REGION \
        AWS_EC2_METADATA_DISABLED AWS_DISABLE_HOST_PREFIX_INJECTION NO_PROXY; do
        printf 'export %s=%q\n' "$name" "${!name:-}" >>"$temporary"
    done
    printf 'export FALKORDB_URL=%q\n' "$FALKORDB_TEST_URL" >>"$temporary"
    printf 'export INVENTORY_LOCAL_SESSION=%q\n' "$destination" >>"$temporary"
    mv "$temporary" "$SESSION_ENV"
    printf 'READY: authenticated local emulator and FalkorDB; session %s\n' "$destination"
    trap 'exit 0' TERM INT
    while :; do sleep 30 & wait "$!" || true; done
    ;;
status)
    if ! is_running; then
        printf 'Not running. Log: %s\n' "$LOG_FILE" >&2
        exit 1
    fi
    if [[ -f "$SESSION_ENV" ]]; then
        source "$SESSION_ENV"
        if [[ -r "$SSL_CERT_FILE" ]] \
            && curl --fail --silent --max-time 5 --cacert "$SSL_CERT_FILE" \
                "$AWS_ENDPOINT_URL/__admin/health" >/dev/null \
            && redis_ping "$FALKORDB_URL"; then
            printf 'READY\nEmulator: %s\nFalkorDB: %s\nSession: %s\n' \
                "$AWS_ENDPOINT_URL" "$FALKORDB_URL" "$INVENTORY_LOCAL_SESSION"
            exit 0
        fi
    fi
    printf 'Not ready. Inspect: scripts/local-stack.sh logs\n' >&2
    exit 1
    ;;
stop)
    if is_running; then
        pid=$(serve_pid)
        kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
        for _ in $(seq 1 30); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 1
        done
    fi
    stop_falkordb
    rm -f "$PID_FILE" "$SESSION_ENV"
    printf 'Stopped local stack. Saved inventory and evidence are retained.\n'
    ;;
cli)
    shift
    require_install
    "$ROOT/scripts/local-stack.sh" status >/dev/null
    clear_aws
    source "$SESSION_ENV"
    exec "$BINARY" "$@"
    ;;
verify)
    require_install
    destination="$STATE_DIR/verification/$(date -u +%Y%m%dT%H%M%S)-$$"
    mkdir -p "$destination"
    "$HARNESS" verify-delivery \
        --binary "$BINARY" --redis-cli "$REDIS_CLI" \
        --database "$destination/inventory.sqlite" \
        --output "$destination/delivery-verification.json" \
        --run-workspace-tests
    ;;
logs)
    tail -n 50 "$LOG_FILE" 2>/dev/null || printf 'No log yet: %s\n' "$LOG_FILE" >&2
    ;;
help | -h | --help)
    printf '%s\n' 'Usage: scripts/local-stack.sh {install|sdk-build|start|status|stop|cli ARGS...|verify|logs}' \
        'start: background loopback AWS emulator + FalkorDB; no sudo or live AWS.' \
        'FalkorDB: FALKORDB_TEST_URL if set, else docker/podman falkordb/falkordb:6.0.4,' \
        '  else FALKORDB_REDIS_SERVER + FALKORDB_MODULE (defaults under the runtime dir).' \
        'cli: invoke the collector using only the verified local emulator environment.' \
        'verify: fresh fixtures, all Rust tests, exact SQLite/export/graph checks and replay.' \
        'sdk-build: compile all SDKs in a separate target directory.' \
        "State: ${STATE_DIR} (override with AWS_INVENTORY_STATE_DIR)." \
        'Services stop when stop runs or on reboot; use start again to recreate fixtures.'
    ;;
*)
    printf 'Unknown command: %s\n' "$1" >&2
    exit 2
    ;;
esac

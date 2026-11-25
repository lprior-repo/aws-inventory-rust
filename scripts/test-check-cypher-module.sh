#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
root=$(mktemp -d)
trap 'rm -rf -- "$root"' EXIT
mkdir -p -- "$root/crates/probe/src"
probe="$root/crates/probe/src/lib.rs"
printf '%s\n' 'pub fn ordinary_rust() {}' > "$probe"
bash "$script_dir/check-cypher-module.sh" "$root"

forms=(
    'redis::cmd("GRAPH.QUERY")'
    'redis::cmd("GRAPH.RO_QUERY")'
    '"MATCH(n) RETURN n"'
    '"OPTIONAL MATCH(n) RETURN n"'
    '"UNWIND rows AS row"'
    '"DETACH DELETE n"'
    '"MERGE(n)"'
    $'"CREATE \\\n(n)"'
    '"match(n) return n"'
    '"RETURN 1"'
    '"SET n.account = value"'
    '"CYPHER key=value"'
)
for form in "${forms[@]}"; do
    probe=$(mktemp "$root/crates/probe/src/form.XXXXXX.rs")
    printf '%s\n' "$form" > "$probe"
    status=0
    bash "$script_dir/check-cypher-module.sh" "$root" > "$root/output" 2>&1 || status=$?
    if [[ "$status" != 1 ]]; then
        printf 'gate missed %q (exit %s)\n' "$form" "$status" >&2
        cat -- "$root/output" >&2
        exit 1
    fi
    rm -- "$probe"
done
printf 'Cypher gate: %s forbidden forms rejected; ordinary Rust accepted.\n' "${#forms[@]}"

#!/usr/bin/env bash
set -euo pipefail

script_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
# An optional root permits isolated regression fixtures without touching the tree.
root=${1:-$script_root}
cd -- "$root"
globs=(--glob '*.rs' --glob '!crates/graph-store/src/**' --glob '!crates/engine/src/graph/**')
while IFS= read -r entry; do
    [[ -z "$entry" || "$entry" == \#* ]] && continue
    path=${entry%% # *}
    reason=${entry#* # }
    if [[ "$path" == "$entry" || -z "$reason" || "$path" != *.rs || "$path" == *[\*\?\[]* ]]; then
        printf 'Invalid Cypher allowlist entry: %s\n' "$entry" >&2
        exit 2
    fi
    globs+=(--glob "!$path")
done < "$script_root/scripts/cypher-allowlist.txt"

# Extract string literals first so Rust's `match (tuple, value)` is not Cypher.
# Single and multiline strings (including escaped line continuations) are checked.
status=0
rg -o -U --no-filename --no-line-number "${globs[@]}" '(?s)"(\\.|[^"\\])*"' . |
    rg -n -i -U '\b(CYPHER(\s|")|GRAPH\.(RO_)?QUERY\b|(OPTIONAL\s+)?MATCH\s*\(\s*([A-Za-z_]\w*\s*)?[:{)]|(CREATE|MERGE)\s*(\\\s*)?\(\s*([A-Za-z_]\w*\s*)?[:{)]|UNWIND\b|DETACH\s+DELETE\b|RETURN\s+1\b|SET\s+[A-Za-z_]\w*\.[A-Za-z_])' || status=$?
case "$status" in
    0) printf '%s\n' 'Cypher must live in the graph-store module or a reasoned per-file allowlist entry.' >&2; exit 1 ;;
    1) exit 0 ;;
    *) exit "$status" ;;
esac

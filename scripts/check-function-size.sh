#!/usr/bin/env bash
set -euo pipefail

# Function-length gate: functions on the new CMDB surfaces must stay under 60 lines.
# Span is measured from the `fn` line through the closing brace. Macro-heavy
# bodies are counted conservatively; the check intentionally fails closed.
MAX_LINES="${MAX_FUNCTION_LINES:-60}"
status=0

check_file() {
    local file="$1"
    case "$file" in
        */tests.rs|*/tests/*|*_tests.rs) return 0 ;;
    esac
    local violations
    violations="$(
        awk -v max="$MAX_LINES" -v file="$file" '
            /^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?(const[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]/ {
                if (active) {
                    if (NR - start >= max) {
                        printf "%s:%d: %d lines: %s\n", file, start, NR - start, name
                    }
                }
                active = 1
                start = NR
                name = $0
                sub(/^[[:space:]]+/, "", name)
                sub(/[[:space:]]*\{.*/, "", name)
                braces = 0
                seen_open = 0
            }
            active {
                line = $0
                opens = gsub(/\{/, "{", line)
                closes = gsub(/\}/, "}", line)
                braces += opens - closes
                if (opens > 0) seen_open = 1
                if (seen_open && braces <= 0) {
                    if (NR - start + 1 >= max) {
                        printf "%s:%d: %d lines: %s\n", file, start, NR - start + 1, name
                    }
                    active = 0
                }
            }
        ' "$file"
    )"
    if [[ -n "$violations" ]]; then
        printf '%s\n' "$violations"
        status=1
    fi
}

paths=(
    crates/harness/src
    crates/aws/src/cloud_control/universal
    crates/engine/src/graph/reference
    src/plan.rs
    src/plan
    crates/cmdb-bench/src
    crates/cmdb-ui/src
    crates/cmdb-view/src
    crates/catalog-tool/src
)
for path in "${paths[@]}"; do
    [[ -e "$path" ]] || continue
    if [[ -f "$path" ]]; then
        check_file "$path"
    else
        while IFS= read -r file; do
            check_file "$file"
        done < <(find "$path" -type f -name '*.rs' | sort)
    fi
done

if [[ "$status" -ne 0 ]]; then
    printf 'function-length gate failed (max %s lines)\n' "$MAX_LINES" >&2
fi
exit "$status"

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
failed=0
count=0
# Reviewed data tables are exempt: one literal row per decision.
is_table() {
    case "$1" in
        crates/catalog-tool/src/kind_map/builtin_operation.rs) return 0 ;;
        crates/catalog-tool/src/kind_map/builtin_api.rs) return 0 ;;
        crates/catalog-tool/src/kind_map/builtin_relation.rs) return 0 ;;
        crates/catalog-tool/src/kind_map/builtin_cfn.rs) return 0 ;;
        crates/catalog-tool/src/classes/deny/builtin.rs) return 0 ;;
    esac
    return 1
}
while IFS= read -r -d '' file; do
    [[ -f "$file" ]] || continue
    is_table "$file" && continue
    lines=$(wc -l < "$file")
    count=$((count + 1))
    if (( lines > 1000 )); then
        printf '%s:%s: exceeds 1000-line Rust source limit\n' "$file" "$lines"
        failed=1
    fi
done < <(git ls-files --cached --others --exclude-standard -z -- '*.rs')
if (( failed )); then
    exit 1
fi
printf 'Rust source size: %s files, all at most 1000 lines\n' "$count"

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
failed=0
count=0
while IFS= read -r -d '' file; do
    [[ -f "$file" ]] || continue
    lines=$(wc -l < "$file")
    count=$((count + 1))
    if (( lines > 300 )); then
        printf '%s:%s: exceeds 300-line Rust source limit\n' "$file" "$lines"
        failed=1
    fi
done < <(git ls-files --cached --others --exclude-standard -z -- '*.rs')
if (( failed )); then
    exit 1
fi
printf 'Rust source size: %s files, all at most 300 lines\n' "$count"

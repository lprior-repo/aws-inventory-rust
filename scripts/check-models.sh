#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
path=.local/aws-models
status=$(git submodule status -- "$path")
if [[ "$status" != " "* ]]; then
    printf 'models are not at the pinned commit: %s\n' "$status" >&2
    exit 1
fi
changed=$(git -C "$path" status --porcelain)
if [[ -n "$changed" ]]; then
    printf 'model files differ from the pinned commit:\n%s\n' "$changed" >&2
    exit 1
fi
count=$(git -C "$path" ls-files 'models/*.json' | wc -l)
printf 'AWS API models: %s files at %s\n' "$count" "$(git -C "$path" rev-parse HEAD)"

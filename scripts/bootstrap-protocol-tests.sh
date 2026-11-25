#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

fixtures=crates/aws-wire/tests/fixtures/protocol-tests
recorded=scripts/protocol-tests/outputs.sha256
converter=scripts/protocol-tests/convert.sh

for tool in git curl unzip; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf 'bootstrap-protocol-tests: %s not found; install it and rerun.\n' "$tool" >&2
        exit 1
    fi
done

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    else
        shasum -a 256 "$1" | cut -d ' ' -f 1
    fi
}

outputs_match() {
    while read -r want name; do
        if [ ! -f "$1/$name" ] || [ "$(sha256_of "$1/$name")" != "$want" ]; then
            return 1
        fi
    done < "$recorded"
}

if outputs_match "$fixtures"; then
    printf 'bootstrap-protocol-tests: fixtures already up to date.\n'
    exit 0
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/bootstrap-protocol-tests.XXXXXX")
trap 'rm -rf -- "$tmp"' EXIT

sh "$converter" "$tmp/out"

if ! outputs_match "$tmp/out"; then
    printf 'bootstrap-protocol-tests: regenerated fixtures do not match %s; nothing was installed.\n' \
        "$recorded" >&2
    exit 1
fi

mkdir -p "$fixtures"
while read -r _ name; do
    cp "$tmp/out/$name" "$fixtures/$name"
done < "$recorded"
printf 'bootstrap-protocol-tests: wrote and verified fixtures in %s.\n' "$fixtures"

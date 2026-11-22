#!/usr/bin/env bash
# Regenerates the seven Smithy protocol-test fixtures that aws-wire tests read.
# The outputs are gitignored; they are never committed.
#
# Idempotent: exits at once when every output exists and matches the digests in
# scripts/protocol-tests/outputs.sha256. Otherwise the converter runs into a
# temporary directory, and the results are copied in only after every digest
# matches. Supported hosts: Linux x86_64, Linux aarch64, Darwin arm64.
#
# Bash 3.2 compatible (macOS /bin/bash): no arrays, mapfile or associative maps.
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

# Succeeds only when every recorded output exists under $1 with its digest.
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

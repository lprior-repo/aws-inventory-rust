#!/bin/sh
# Offline-build fixture maintenance only; never invoked by the product build.
# Source: https://github.com/smithy-lang/smithy, released 2026-11-12.
# Smithy 1.75.0, bundled Corretto 25.0.5+10-LTS; canonicalizer jq-1.8.2.
set -eu
# Recorded ancillary tools: git 2.56.0, curl 8.23.0, UnZip 6.00,
# GNU coreutils sha256sum 9.12. Only Smithy and jq determine output bytes.
# Release discovery (not a conversion input): GitHub releases/latest response,
# fetched 2026-11-13, SHA-256:
# 7d8e5a60f227116cbd5fcfe606ded66eb3d10ad98ebd518b93ce77c4360ddcae
export LC_ALL=C TZ=UTC

TAG=1.75.0
COMMIT=981f95a2cae567752b70749c1f04502c076a7a63
CLI_SHA256=c8d448426061586a0afb93830151838cb92a73e45d9d3733c3367ca905eb32c1
REPOSITORY=https://github.com/smithy-lang/smithy
ARCHIVE=smithy-cli-linux-x86_64.zip

if [ "$#" -gt 1 ]; then
    printf 'Usage: sh %s [output-directory]\n' "$0" >&2
    exit 2
fi
if [ "$(jq --version)" != jq-1.8.2 ]; then
    printf 'jq-1.8.2 is required for byte-identical number serialization.\n' >&2
    exit 1
fi
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
output=${1:-"$script_dir/../../crates/aws-wire/tests/fixtures/protocol-tests"}
mkdir -p -- "$output"
output=$(CDPATH= cd -- "$output" && pwd)
tmp=$(mktemp -d /tmp/smithy-protocol-fixtures.XXXXXX)
trap 'rm -rf -- "$tmp"' 0
trap 'exit 1' HUP INT TERM
cd "$tmp"

# The tag identifies the release; the immutable commit guards against retagging.
git clone --quiet --depth 1 --branch "$TAG" "$REPOSITORY" source
if [ "$(git -C source rev-parse HEAD)" != "$COMMIT" ]; then
    printf 'Smithy release commit does not match the pinned commit.\n' >&2
    exit 1
fi
curl --fail --location --silent --show-error \
    "$REPOSITORY/releases/download/$TAG/$ARCHIVE" --output "$ARCHIVE"
printf '%s  %s\n' "$CLI_SHA256" "$ARCHIVE" | sha256sum --check -
unzip -q "$ARCHIVE" -d cli
smithy="$tmp/cli/smithy-cli-linux-x86_64/bin/smithy"
if [ "$("$smithy" --version)" != "$TAG" ]; then
    printf 'Unexpected Smithy CLI version.\n' >&2
    exit 1
fi

aws=source/smithy-aws-protocol-tests/model
generic=source/smithy-protocol-tests/model
for protocol in awsJson1_0 awsJson1_1 awsQuery ec2Query restJson1 restXml rpcv2Cbor; do
    set -- \
        source/smithy-aws-traits/src/main/resources/META-INF/smithy \
        source/smithy-protocol-test-traits/src/main/resources/META-INF/smithy \
        source/smithy-protocol-traits/src/main/resources/META-INF/smithy \
        source/smithy-validation-model/model \
        "$aws/shared-types.smithy" "$aws/aws-config.smithy" "$aws/$protocol"
    case "$protocol" in
        restXml) set -- "$@" "$aws/restXmlWithNamespace" ;;
        rpcv2Cbor)
            set -- "$@" "$generic/shared-types.smithy" "$generic/rpcv2Cbor" \
                "$generic"/v2/*.smithy "$generic/v2/rpcv2cbor"
            ;;
    esac
    # Keep the complete AST, prelude, mixins, traits, metadata and test cases.
    # Separate commands ensure a failing Smithy process cannot hide in a pipe.
    "$smithy" ast --no-config --include-prelude --output "$tmp/build" "$@" \
        > "$protocol.ast.json"
    jq --sort-keys . "$protocol.ast.json" > "$protocol.json"
    jq -e . "$protocol.json" > /dev/null
    if [ "$(wc -c < "$protocol.json")" -gt 3000000 ]; then
        printf '%s exceeds the 3 MB fixture limit.\n' "$protocol" >&2
        exit 1
    fi
done

for protocol in awsJson1_0 awsJson1_1 awsQuery ec2Query restJson1 restXml rpcv2Cbor; do
    cp -- "$protocol.json" "$output/$protocol.json"
done
cd "$output"
sha256sum awsJson1_0.json awsJson1_1.json awsQuery.json ec2Query.json \
    restJson1.json restXml.json rpcv2Cbor.json

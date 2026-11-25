#!/bin/sh
set -eu
export LC_ALL=C TZ=UTC

TAG=1.75.0
COMMIT=981f95a2cae567752b70749c1f04502c076a7a63
REPOSITORY=https://github.com/smithy-lang/smithy
JQ_REPOSITORY=https://github.com/jqlang/jq/releases/download/jq-1.8.2

case "$(uname -s)/$(uname -m)" in
    Linux/x86_64 | Linux/amd64)
        platform=linux-x86_64
        cli_sha256=c8d448426061586a0afb93830151838cb92a73e45d9d3733c3367ca905eb32c1
        jq_asset=jq-linux-amd64
        jq_sha256=b1c22172dd303f3be49e935aa56aa48a8b7a46e0bc838b4997d3bb451495870f
        ;;
    Linux/aarch64 | Linux/arm64)
        platform=linux-aarch64
        cli_sha256=8561f72386464ac8cfb0ccd3e864a59199e134f26d8ebf6bd264eebe1aa58098
        jq_asset=jq-linux-arm64
        jq_sha256=8b85c817833814ddca00a144c33705546355afccf0cf39b188f3cdb48b852309
        ;;
    Darwin/arm64)
        platform=darwin-aarch64
        cli_sha256=c6d32931b8eaf5e93c159c870387b4e6d8c97363c8c6bb09061291c93cdc9181
        jq_asset=jq-macos-arm64
        jq_sha256=2d75340ba57a4b4b4c8708a21c2dc8e958a48aaa8bba13b27f77f6e4c0eca07e
        ;;
    *)
        printf 'Unsupported platform %s/%s; supported: Linux x86_64, Linux aarch64, Darwin arm64.\n' \
            "$(uname -s)" "$(uname -m)" >&2
        exit 1
        ;;
esac
ARCHIVE=smithy-cli-$platform.zip

if [ "$#" -gt 1 ]; then
    printf 'Usage: sh %s [output-directory]\n' "$0" >&2
    exit 2
fi

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    else
        shasum -a 256 "$1" | cut -d ' ' -f 1
    fi
}

verify_sha256() {
    actual=$(sha256_of "$1")
    if [ "$actual" != "$2" ]; then
        printf 'SHA-256 mismatch for %s: expected %s, got %s.\n' "$1" "$2" "$actual" >&2
        exit 1
    fi
}

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
output=${1:-"$script_dir/../../crates/aws-wire/tests/fixtures/protocol-tests"}
mkdir -p -- "$output"
output=$(CDPATH= cd -- "$output" && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/smithy-protocol-fixtures.XXXXXX")
trap 'rm -rf -- "$tmp"' 0
trap 'exit 1' HUP INT TERM

mkdir -p -- "$tmp/tools"
curl --fail --location --silent --show-error \
    "$JQ_REPOSITORY/$jq_asset" --output "$tmp/tools/jq"
verify_sha256 "$tmp/tools/jq" "$jq_sha256"
chmod +x "$tmp/tools/jq"
jq="$tmp/tools/jq"
if [ "$("$jq" --version)" != jq-1.8.2 ]; then
    printf 'jq-1.8.2 is required for byte-identical number serialization.\n' >&2
    exit 1
fi

cd "$tmp"

git clone --quiet --depth 1 --branch "$TAG" "$REPOSITORY" source
if [ "$(git -C source rev-parse HEAD)" != "$COMMIT" ]; then
    printf 'Smithy release commit does not match the pinned commit.\n' >&2
    exit 1
fi
curl --fail --location --silent --show-error \
    "$REPOSITORY/releases/download/$TAG/$ARCHIVE" --output "$ARCHIVE"
verify_sha256 "$ARCHIVE" "$cli_sha256"
unzip -q "$ARCHIVE" -d cli
smithy="$tmp/cli/smithy-cli-$platform/bin/smithy"
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
    "$smithy" ast --no-config --include-prelude --output "$tmp/build" "$@" \
        > "$protocol.ast.json"
    "$jq" --sort-keys . "$protocol.ast.json" > "$protocol.json"
    "$jq" -e . "$protocol.json" > /dev/null
    size=$(wc -c < "$protocol.json" | tr -d ' ')
    if [ "$size" -gt 3000000 ]; then
        printf '%s exceeds the 3 MB fixture limit.\n' "$protocol" >&2
        exit 1
    fi
done

for protocol in awsJson1_0 awsJson1_1 awsQuery ec2Query restJson1 restXml rpcv2Cbor; do
    cp -- "$protocol.json" "$output/$protocol.json"
done
cd "$output"
for protocol in awsJson1_0 awsJson1_1 awsQuery ec2Query restJson1 restXml rpcv2Cbor; do
    printf '%s  %s.json\n' "$(sha256_of "$protocol.json")" "$protocol"
done

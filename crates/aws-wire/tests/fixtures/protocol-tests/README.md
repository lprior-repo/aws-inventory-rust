# Published Smithy protocol fixtures

These seven files are **complete JSON ASTs**, not a reduced fixture schema. They
are offline test oracles for M2.1a–c and M2.4a–d, never product data or build inputs.
Each includes operation IDs/names, service protocol traits, input/output/error
shapes, shared shapes, trait definitions and the Smithy prelude. Mixins, XML
namespaces, timestamp settings, metadata, `appliesTo`, and test-case values remain
as emitted by Smithy. The only postprocessing is `jq --sort-keys .`: sorted object
keys, two-space indentation, UTF-8 and LF line endings; arrays are not reordered.
No cases are filtered, rewritten or synthesized.

## Provenance

- Source: <https://github.com/smithy-lang/smithy>.
- Released tag: **1.75.0**, published **2026-11-12**; latest stable release returned
  by GitHub's `releases/latest` API when queried on **2026-11-13**.
- Immutable source commit: `981f95a2cae567752b70749c1f04502c076a7a63`.
- Conversion/evidence date: **2026-11-13**, Linux x86-64.
- Smithy CLI: **1.75.0**, released Linux x86-64 distribution. Its bundled runtime
  is **Corretto 25.0.5+10-LTS**; a separately installed JVM is not used.
- Canonicalizer: **jq-1.8.2**, required by the script, including its preservation
  of numeric literals. Changing jq versions requires deliberate fixture review.
- Other observed tools: git **2.56.0**, curl **8.23.0**, UnZip **6.00**,
  GNU coreutils sha256sum **9.12**. They fetch/check/extract inputs, not render ASTs.

The converter downloads exactly one non-git conversion artifact:

```text
https://github.com/smithy-lang/smithy/releases/download/1.75.0/smithy-cli-linux-x86_64.zip
33528083 bytes
SHA-256 c8d448426061586a0afb93830151838cb92a73e45d9d3733c3367ca905eb32c1
```

The script checks that digest before extraction/execution and checks the source
commit after cloning the release tag. Git objects are pinned by the commit, not
by an additional mutable tarball. No Maven resolution or dependency downloads are
needed: trait/model dependencies come from the same pinned source checkout.
The discovery-only API response (not a conversion input) was fetched with:

```sh
curl --fail --location --silent --show-error \
  https://api.github.com/repos/smithy-lang/smithy/releases/latest --output release.json
sha256sum release.json
# 7d8e5a60f227116cbd5fcfe606ded66eb3d10ad98ebd518b93ce77c4360ddcae
```

### Source selection at the pinned commit

Paths below are relative to the source checkout. `A` means
`smithy-aws-protocol-tests/model`; `P` means `smithy-protocol-tests/model`.

| Fixture | Suite source paths |
| --- | --- |
| `awsJson1_0.json` | `A/awsJson1_0/` |
| `awsJson1_1.json` | `A/awsJson1_1/` |
| `awsQuery.json` | `A/awsQuery/` |
| `ec2Query.json` | `A/ec2Query/` |
| `restJson1.json` | `A/restJson1/` (including nested service and malformed-request suites) |
| `restXml.json` | `A/restXml/` and complementary `A/restXmlWithNamespace/` |
| `rpcv2Cbor.json` | `P/rpcv2Cbor/`, AWS query-compatibility cases in `A/rpcv2Cbor/`, and `P/v2/rpcv2cbor/` |

Every AST also loads `A/shared-types.smithy`, `A/aws-config.smithy`,
`smithy-validation-model/model/`, and the `src/main/resources/META-INF/smithy/`
directories of `smithy-aws-traits`, `smithy-protocol-test-traits` and
`smithy-protocol-traits`. CBOR additionally loads `P/shared-types.smithy` and
`P/v2/*.smithy` (shared corpus declarations, not the sibling rpcv2Json tests).
Thus its full AST includes dependency-only mixin operations with no HTTP cases.
The unrelated eighth protocol, rpcv2Json, is not converted.

## Reproduce

The seven JSON files are gitignored by `.gitignore` in this directory and are
not tracked. From the repository root, with network access and the tools above:

```sh
sh scripts/bootstrap-protocol-tests.sh
```

The bootstrap runs `scripts/protocol-tests/convert.sh` and installs the outputs
only if their SHA-256 digests match `scripts/protocol-tests/outputs.sha256`; it
is idempotent. The converter supports Linux x86-64 and aarch64 only. It exits
on other platforms, so macOS is unsupported. To write the fixtures to a separate
directory instead, from any working directory:

```sh
sh /absolute/path/to/scripts/protocol-tests/convert.sh /tmp/my-protocol-fixtures
```

The script contains the exact rerunnable commands, pinned versions and artifact
digest. It creates a fresh `/tmp/smithy-protocol-fixtures.XXXXXX` directory for
**each invocation**, clones the pinned release, downloads/checks/extracts the CLI,
and runs, for each selected suite and dependency set:

```sh
"$smithy" ast --no-config --include-prelude --output "$tmp/build" "$@" \
  > "$protocol.ast.json"
jq --sort-keys . "$protocol.ast.json" > "$protocol.json"
jq -e . "$protocol.json" > /dev/null
```

No unknown-trait bypass or flattening is used. Smithy failures are checked before
jq runs (no failure-masking pipeline). All seven conversions must succeed and
be at most 3,000,000 bytes each before any destination fixture is copied.
The temporary checkout, CLI and intermediate ASTs are removed on exit. Only the
seven JSON files are replaced; this README is maintained separately. Nothing is
added to Cargo, a build script, or `scripts/verify.sh`.

### Observed two-run byte reproduction

Two independent invocations downloaded fresh inputs and converted in separate
clean temporary directories. Run 1 wrote `/tmp/aws-m21fx-run1.YTFBOi/fixtures`;
run 2 was launched from `/tmp/aws-m21fx-run2.NehXD6` and wrote these committed
fixtures. Both completed successfully, including every `jq -e .` check.
`cmp` of the complete digest outputs and of each pair of JSON files succeeded.

Run 1 SHA-256:

```text
313abe1d8f8e4ccdda493f98ea2dd9f0e6551b913de35246b84def7139d1deee  awsJson1_0.json
e97baaff510f3ad879accf24dc69db2725b80fb70ce1319db47766cbd92d4c33  awsJson1_1.json
98ec0420f78aa66f64b12128ea37eda148fac69e9eef9e7f8ad98f48cc7f0245  awsQuery.json
65056764e96839b980366c1690fb62b7f90d8f404ae1827596f76c09264bff63  ec2Query.json
1d9dde5a1be7db3c56e81e8109aee928f2a606a5147ef1f0d1d3dde1ff157dfb  restJson1.json
ebcf156e461a0d53efb7d3257187f9f83b3f058836eaea32ac6b11708793e6d5  restXml.json
432d6ef6ab2b0d4d9c65965615ca068937832c3969ec03da5a2c2f171826b93a  rpcv2Cbor.json
```

Run 2 SHA-256 (the committed files):

```text
313abe1d8f8e4ccdda493f98ea2dd9f0e6551b913de35246b84def7139d1deee  awsJson1_0.json
e97baaff510f3ad879accf24dc69db2725b80fb70ce1319db47766cbd92d4c33  awsJson1_1.json
98ec0420f78aa66f64b12128ea37eda148fac69e9eef9e7f8ad98f48cc7f0245  awsQuery.json
65056764e96839b980366c1690fb62b7f90d8f404ae1827596f76c09264bff63  ec2Query.json
1d9dde5a1be7db3c56e81e8109aee928f2a606a5147ef1f0d1d3dde1ff157dfb  restJson1.json
ebcf156e461a0d53efb7d3257187f9f83b3f058836eaea32ac6b11708793e6d5  restXml.json
432d6ef6ab2b0d4d9c65965615ca068937832c3969ec03da5a2c2f171826b93a  rpcv2Cbor.json
```

## Case counts and sizes

Each triple is **client-only / server-only / both**. Missing `appliesTo` means
both; the fixtures retain absence rather than inserting a value. Requests and
successful responses are counted on operation shapes; error responses are
counted on shapes with `smithy.api#error`, once per shape case, not multiplied by
the number of operations referencing that error.

| Protocol | Request cases | Response cases | Error cases | JSON bytes |
| --- | ---: | ---: | ---: | ---: |
| awsJson1_0 | 7 / 4 / 22 | 9 / 2 / 16 | 11 / 0 / 5 | 316,372 |
| awsJson1_1 | 2 / 3 / 55 | 7 / 1 / 41 | 11 / 0 / 3 | 331,975 |
| awsQuery | 3 / 0 / 35 | 8 / 0 / 28 | 0 / 0 / 3 | 316,058 |
| ec2Query | 3 / 0 / 27 | 3 / 0 / 24 | 0 / 0 / 2 | 293,042 |
| restJson1 | 22 / 17 / 120 | 13 / 8 / 81 | 11 / 0 / 3 | 998,765 |
| restXml | 12 / 4 / 97 | 9 / 2 / 72 | 0 / 0 / 3 | 489,113 |
| rpcv2Cbor | 6 / 14 / 25 | 18 / 2 / 22 | 0 / 0 / 5 | 484,446 |

Totals: **478 request + 366 response + 57 error = 901 HTTP cases**;
**3,229,771 JSON bytes**. Every individual file is below 3 MB.
These counts and sizes were also queried directly from the committed files.

Recount from this directory with these jq expressions:

```sh
for file in *.json; do
  jq -e . "$file" > /dev/null
  jq --arg file "$file" '
    def counts:
      . as $cases | ["client", "server", "both"] |
      map(. as $side |
        [$cases[] | select((.appliesTo // "both") == $side)] | length);
    [.shapes[] | select(.type == "operation") |
      .traits["smithy.test#httpRequestTests"][]?] as $request |
    [.shapes[] | select(.type == "operation") |
      .traits["smithy.test#httpResponseTests"][]?] as $response |
    [.shapes[] | select(.traits["smithy.api#error"] != null) |
      .traits["smithy.test#httpResponseTests"][]?] as $error |
    {file: $file, request: ($request | counts),
     response: ($response | counts), error: ($error | counts)}
  ' "$file"
done
wc -c *.json
```

## Harness features and scope

All seven suites use `appliesTo`, `body`, `bodyMediaType`, `code`,
`documentation`, `headers`, `id`, `method`, `params`, `protocol` and `uri` across
their request/response/error cases. Additional observed top-level case fields:

| Protocol | Additional fields |
| --- | --- |
| awsJson1_0 | `host`, `resolvedHost`, `tags`, `vendorParams`, `vendorParamsShape` |
| awsJson1_1 | `host`, `resolvedHost`, `requireHeaders` |
| awsQuery | `host`, `resolvedHost`, `requireHeaders`, `vendorParams`, `vendorParamsShape` |
| ec2Query | `host`, `resolvedHost`, `requireHeaders`, `tags`, `vendorParams`, `vendorParamsShape` |
| restJson1 | `forbidHeaders`, `forbidQueryParams`, `host`, `queryParams`, `requireHeaders`, `resolvedHost`, `tags` |
| restXml | `forbidHeaders`, `forbidQueryParams`, `host`, `queryParams`, `requireHeaders`, `resolvedHost`, `vendorParams`, `vendorParamsShape` |
| rpcv2Cbor | `forbidHeaders`, `requireHeaders`, `tags`, `vendorParams`, `vendorParamsShape` |

`authScheme` and `requireQueryParams` do **not** appear as top-level HTTP case
fields in these pinned suites. Do not infer their absence from the protocols in
general. Enumerate actual case features with:

```sh
for file in *.json; do
  jq '{features: ([.shapes[] | .traits |
    .["smithy.test#httpRequestTests"][]?,
    .["smithy.test#httpResponseTests"][]? | keys[]] | unique),
    malformedRequests: ([.shapes[] |
      .traits["smithy.test#httpMalformedRequestTests"][]?] | length),
    eventStreams: ([.shapes[] |
      .traits["smithy.test#eventStreamTests"][]?] | length)}' "$file"
done
```

Additional traits are retained but are not included in the 901 HTTP cases:
`restJson1` has **191 malformed-request cases** and **102 event-stream cases**;
`rpcv2Cbor` has **22 event-stream cases**; the other fixtures have neither.
A read-only buffered client does not implement server-only cases, inbound
malformed-request validation, or event-stream framing. Those are explicit harness
scope decisions, not missing fixtures. Cases are protocol conformance examples,
including synthetic write operations; consuming them must not authorize product
write calls. Server-only cases remain available for other harnesses. Client-only
and both-side cases apply to client request serialization/response decoding.
CBOR bodies can be base64 strings selected by `bodyMediaType`; vendor parameter
shape references, host-prefix expectations, header/query restrictions, default
values, unions, recursive shapes and XML namespace/timestamp traits must not be
silently discarded by consumers.

## Licence and NOTICE

Fixtures are derived from Smithy under **Apache-2.0**, not this project's MIT
licence. Source models are unchanged except IDL-to-AST conversion and JSON key
ordering. The converter is this project's maintenance tooling.

Upstream licence at the pinned commit:
<https://github.com/smithy-lang/smithy/blob/981f95a2cae567752b70749c1f04502c076a7a63/LICENSE>
(SHA-256 `09e8a9bcec8067104652c168685ab0931e7868f9c8284b66f5ae6edae5f1130b`).
Upstream NOTICE:
<https://github.com/smithy-lang/smithy/blob/981f95a2cae567752b70749c1f04502c076a7a63/NOTICE>
(SHA-256 `2f4eba28c182e0a323ca533598bcca15488326628e7a6fadd230af1c10270ba5`),
reproduced below:

```text
Smithy
Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.

**********************
THIRD PARTY COMPONENTS
**********************

This software includes third party software subject to the following copyrights.

minimal-json
Copyright (c) 2013, 2014 EclipseSource
https://github.com/ralfstx/minimal-json/blob/master/LICENSE


Apache Commons Text
Copyright 2014-2018 The Apache Software Foundation

This product includes software developed at
The Apache Software Foundation (http://www.apache.org/).


Apache Commons Text
Copyright 2014-2018 The Apache Software Foundation

This product includes software developed at
The Apache Software Foundation (http://www.apache.org/).


Elephant Bird (toSnakeCase method in the codegen module)
Copyright 2014-2018 The Apache Software Foundation

This product includes software developed at
The Apache Software Foundation (http://www.apache.org/).


sphinxcontrib-redirects (redrects package in docs/smithy)
Copyright (c) 2017 Stephen Finucane.
```

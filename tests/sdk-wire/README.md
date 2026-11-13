# Resource Explorer SDK protocol fixtures

This isolated test-only crate checks the unchanged, pinned AWS Rust SDK against
literal in-memory responses using the official
`aws_smithy_http_client::test_util::StaticReplayClient`. It does not run an HTTP
server, contact AWS, read an ambient credential chain, or modify emulator output.
These tests are **protocol fixture tests, not successful emulator integration
results**.

The production workspace and this crate have separate Cargo workspaces and target
directories. Enabling Smithy `test-util` here does not rebuild the large EC2 SDK
with test-only features.

## What is checked

| Operation | Modeled representation accepted | Numeric representation rejected |
| --- | --- | --- |
| GetIndex | CreatedAt and LastUpdatedAt ISO 8601 strings, including an explicit UTC offset | Each field is changed to a numeric epoch independently |
| GetView | View.LastUpdatedAt ISO 8601 string | View.LastUpdatedAt numeric epoch |
| ListResources | Resource and resource-property LastReportedAt strings, nonempty resource and next token | Each timestamp level is changed to a numeric epoch independently |

Six tests cover three successful responses and five independently failing numeric
fields. Positive fixtures are constructed from the published response shapes;
negative variants reproduce the numeric token type observed in emulator evidence.
They are not represented as captured production or emulator responses.

Every request uses a synthetic static credential provider, a non-routable fixture
endpoint, a single SDK attempt, and a two-second operation bound. The official
replay transport intercepts the request entirely in memory. Tests verify the
request method, URI, JSON inputs, one consumed replay request, decoded timestamps
and identities, or an HTTP-200 output-deserialization failure without a modeled
service error code. SDK source and parser behavior remain unchanged.

## Run

From the repository root:

```sh
CARGO_TARGET_DIR="$PWD/target/sdk-wire" cargo test --manifest-path tests/sdk-wire/Cargo.toml --locked --offline -j1
CARGO_TARGET_DIR="$PWD/target/sdk-wire" cargo clippy --manifest-path tests/sdk-wire/Cargo.toml --all-targets --locked --offline -j1 -- -D warnings
```

The test utility adds the official `aws-smithy-protocol-test` dependency. Fetch the
isolated lockfile's dependencies before using `--offline` on a clean machine.
The SDK and direct Smithy dependency versions match the root lockfile snapshot.

## Evidence and sources

The production SDK uses `Format::DateTimeWithOffset` for all five checked fields:
`aws-sdk-resourceexplorer2 1.126.0` modules
`protocol_serde/shape_get_index.rs`, `shape_view.rs`, `shape_resource.rs`, and
`shape_resource_property.rs`.

- [AWS GetIndex response syntax](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetIndex.html)
- [AWS GetView response syntax](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetView.html)
- [AWS ListResources response syntax](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html)
- [Official Smithy StaticReplayClient](https://docs.rs/aws-smithy-http-client/1.5.2/aws_smithy_http_client/test_util/struct.StaticReplayClient.html)
- [Emulator compatibility analysis](../../docs/EMULATOR_TESTING.md)
- [Observed emulator timestamp evidence](../../artifacts/emulator-resource-explorer-wire-check.json)

The final two links refer to repository evidence separately from these constructed
fixtures. A successful fixture test must never replace an emulator failure in the
integration coverage report.

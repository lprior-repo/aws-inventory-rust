# AWS property tests, retry ownership and collector oracles

> Status: Baseline being replaced. These tests cover the typed-SDK adapters and
> their one-attempt SDK configuration, which go with the adapters in ROADMAP M5
> and M9.5; the rule they protect, one retry owner, is kept (VISION 5.3). See
> [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

The AWS tests exercise generated error metadata, retry directives, pagination histories, discovery identities and SDK configuration. A separate, explicitly ignored integration target runs the real public collector against a prepared local emulator and compares its output with a verified snapshot. Passing helper tests alone does not demonstrate that a collector still performs its requests or emits its resources.

This document describes the test contracts. A mutation percentage must come from completed, reproducible campaign receipts for a frozen candidate set; the presence or count of these tests does not establish 95% mutation coverage.

## Generated tests

The dependency is exactly Proptest 1.11.0 with its `std` feature. The property suites retain shrinking and normal failure persistence. Each module fixes its random seed, bounds its input sizes and uses 128 generated cases per property, except the service-configuration property, which uses 32. Seven deterministic tests cover exact memory/identity boundaries, every named service selector and independent source/page prerequisites. Together these modules contain 66 generated properties and seven deterministic contracts.

| Source | Tests | Contract |
| --- | ---: | --- |
| `crates/aws/src/common/property_tests.rs` | 26 generated + 1 exact boundary | Retry error classification, integer and HTTP-date directives, repeated-header precedence, safe diagnostics, page/child limits, token cycles and the continuation-history byte ceiling |
| `crates/aws/src/discovery/pages/property_tests.rs` | 12 generated + 2 exact boundaries | Stable first observation per identity; complete terminal pages at exact limits; incomplete continuations at limits; atomic rejection of oversized pages; UTF-8 byte limits; empty pages; retained observations after denial, timeout or cancellation |
| `crates/aws/src/discovery/property_tests.rs` | 12 generated | Role/account identity, independent role-path limits, secret-safe diagnostics, bounded credential deadlines, failure-report capacity, merge accounting, explicit source selection and inherited SDK configuration |
| `crates/aws/src/supplemental/property_tests.rs` | 7 generated | Native-kind mapping, source-type normalization, bounded fallback identifiers, exact ELBv2 ARN arity and rejection of Classic/contradictory identities |
| `crates/aws/src/options_property_tests.rs` | 9 generated + 4 exact tables | Inclusive independent region/selector limits, every native selector and alias, order-independent unions, source prerequisites, Cloud Control opt-in, optional-text byte/NUL/empty checks and region/page grammar |

Run these suites without AWS credentials or an emulator:

```sh
cargo test -p inventory-aws --lib property_tests --locked -j1
```

The pagination properties supply finite callback implementations and assert immediately on any unexpected request or cursor after the generated fixture ends. They do not return a second error into a potentially broken failure handler. Cancellation checks first assert the real cursor's terminal cancellation transition, then run the real driver and verify that no subsequent callback runs and preceding observations remain in an explicitly partial report. The terminal assertion also prevents a deleted failure transition from making that test spin synchronously without another callback. Paused Tokio time is used for these pure, injected async drivers; they do not start SQLx workers or HTTP clients.

The classification properties construct official Smithy typed errors. They verify that modeled throttling/transient metadata is honored, authorization and unsupported operations remain terminal, and HTTP 200 decode errors do not become successful resource observations. Server delay tests distinguish integer seconds, AWS integer milliseconds and HTTP dates with an injected clock. Oversized, invalid and non-UTF-8 directives cannot be hidden by a shorter valid header.

The known-kind property visits every explicitly listed mapping on every generated case. The ELB properties derive independent valid and invalid ARN examples from family grammar; malformed component counts, empty components, wrong services, wrong regions/accounts and inconsistent resource types must not promote a resource to an ELBv2 kind.

### Admission regressions found by the diagnostic campaign

The original emulator snapshot used the `all` selector, so it could verify complete fixture output while a broken individual selector remained undetected. The new selector table asserts each supported native spelling separately, including ECS's task-definition expansion and each EC2, DynamoDB and load-balancing alias. Generated selections assert exact collector unions, ordering/duplicate independence, and rejection of an unknown selector even beside `all`. Supplemental selectors must be enabled by their own source or explicit discovery setting; an unrelated source does not satisfy that prerequisite.

The option properties independently exercise 63/64/65/66 region entries and 31/32/33/34 service entries, with all other inputs valid. Exactly 64 regions and 32 selectors remain accepted together. Optional profile, Config aggregator and Resource Explorer view values are tested at zero, one, 2,047, 2,048, 2,049 and 2,050 bytes with ASCII and multibyte characters; NUL rejection is tested separately from length and emptiness. Explicit resource-type selection must require the Cloud Control flag, while a valid enabled selection remains accepted. These tests call the actual admission function without loading profiles or credentials.

The diagnostic AWS campaign was deliberately stopped after concurrent compilers were killed by resource pressure. Its preserved audit, `artifacts/mutation/aws-v04/interruption-audit.json`, contains 33 completed diagnostic observations: 20 caught, 12 missed and one unresolved compiler SIGKILL, with 1,486 candidates untested or interrupted. The stopped wrapper did not complete its final fixture receipt, so these observations are not a passing campaign or a final mutation score. The interrupted clone was restored and its full source digest verified. All 12 observed misses are mapped to the new contracts in `artifacts/mutation/aws-v04-survivor-plan.json`. A fresh serial campaign must establish which faults the expanded tests detect; the old outcomes are not reused across the changed source/test closure.

## Retry ownership is explicit

The AWS SDK has built-in retry strategies. This application deliberately configures **one SDK attempt per explicit inventory operation** and lets `ApiExecutor` own retries and admission. AWS documents that `max_attempts = 1` disables SDK retries, and that explicit configuration has precedence over environment/config-file defaults. See [AWS Rust retry configuration](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/retries.html) and [AWS retry settings and precedence](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html).

This keeps every explicit inventory attempt behind the same concurrency, pacing, timeout and cancellation boundary. Enabling both the SDK retry loop and the executor retry loop would allow multiple wire attempts beneath one admitted and counted executor attempt. The detailed pinned-runtime analysis, including the constraints on a future migration to SDK-owned retries, is in [SDK_RETRY_AUDIT.md](SDK_RETRY_AUDIT.md).

The source audit found these three configuration boundaries:

| Boundary | Enforced configuration | Consumers |
| --- | --- | --- |
| `crates/aws/src/lib.rs`, public scan loader | `RetryConfig::standard().with_max_attempts(1)` | Initial STS identity, region discovery and all regional/global inventory contexts |
| `crates/aws/src/discovery.rs`, standalone discovery loader | The same single-attempt setting | STS identity, Organizations, Config aggregator discovery and Resource Explorer discovery |
| `crates/aws/src/discovery.rs`, `assume_role_config` | Replaces a caller's arbitrary retry setting with one attempt, marks its explicit origin, and preserves its region/endpoint | Returned inventory configuration and the official nested STS AssumeRole provider configuration |

Every explicit production SDK client is constructed from one of these shared configurations. Regional contexts, partition-global contexts and per-bucket S3 contexts change only their region through `SdkConfig::to_builder()`. They do not introduce another retry configuration.

The executable configuration property supplies explicit synthetic credentials, a fixed clock and a loopback endpoint. It varies the caller's original attempt setting, region and configuration origin, calls the real lazy `assume_role_config` helper, and checks all 15 official service configuration builders: STS, EC2, DynamoDB, Lambda, ECS, EKS, ELBv2, Config, Resource Explorer, Cloud Control, S3, IAM, Route53, CloudFront and Organizations. Both the returned configuration and a region-adjusted clone must retain one attempt. The caller's original configuration must remain unchanged. Lazy provider construction must leave executor attempt and in-flight counts at zero.

This regression verifies the real SDK conversion used by `Client::new`; it does not execute every production loader against ambient AWS profiles. The latter configuration boundary is audited in source and exercised through the separately guarded emulator scans. These are distinct forms of evidence.

### A real configuration defect found by the property

The first expanded property run passed 59 of 60 tests. The configuration property found that a manually built shared SDK configuration could produce a DynamoDB service configuration with four attempts even after `assume_role_config` had written a shared limit of one. The failing input shrank to one original attempt and `us-east-1`.

The pinned `aws-sdk-dynamodb 1.139.0` service conversion applies its November 10 behavior defaults when the shared retry setting has unknown origin. `SdkConfig::Builder::retry_config` writes the value but does not establish its origin. The application's ordinary `aws-config` loaders already mark explicit retry settings as programmatic shared configuration, so this finding concerns the public AssumeRole helper when supplied an otherwise valid manually built configuration; it is not evidence that the ordinary CLI previously retried DynamoDB four times.

The production fix records `Origin::shared_config()` when the helper writes its own explicit retry setting. This is the same provenance used by the official loader and preserves the existing client construction path. The property now varies unknown, environment, profile, IMDS and explicit origins and constructs every service configuration before checking the full vector of attempt limits. All 60 AWS property/boundary tests passed after this fix. The original failing Proptest seed remains in `crates/aws/proptest-regressions/discovery/property_tests.txt`; the pre-fix failure output is retained in `artifacts/aws-properties-single-attempt-regression-v04.log`.

### Credential-provider traffic

Credential refresh is owned by the official SDK provider/cache. The explicit AssumeRole provider receives the bounded single-attempt STS configuration, with operation/attempt/read deadlines no larger than 30 seconds and a connection deadline no larger than 10 seconds. Stricter caller deadlines are preserved. Source-provider behavior, such as SSO, process credentials or a further profile-based role chain, remains a separate SDK concern; it is not silently included in `ApiExecutor` request statistics.

The provider first obtains source credentials and then calls STS when credentials are requested, as documented by the [official AssumeRoleProvider API](https://docs.rs/aws-config/latest/aws_config/sts/struct.AssumeRoleProvider.html). The source/pinned-runtime distinction matters: tests do not assert that arbitrary source providers perform zero network traffic during a later credential fetch. No role, policy or trust relationship is created by configuration or discovery.

## Real collector snapshot tests

`crates/aws/tests/emulator_snapshot.rs` defines four case-specific tests:

| Cargo test name | Collection |
| --- | --- |
| `emulator_snapshot_native` | All native collectors |
| `emulator_snapshot_sources` | Config/Resource Explorer source discovery and collection, retaining the pinned emulator's partial failures |
| `emulator_snapshot_cloudcontrol` | The complete registered Cloud Control catalog, including the three known unsupported namespaces |
| `emulator_snapshot_assumedrole` | IAM collection through an explicitly supplied, same-account existing role |

Each test is ignored during an ordinary Cargo test run with the reason `requires explicitly prepared local emulator and verified frozen snapshot`. Explicitly selecting ignored tests without the prepared environment is a hard failure. There is no successful early return for missing credentials, endpoint or expected data.

The environment must identify a verified `https://127.0.0.1:<port>` endpoint, account `000000000000`, region `us-east-1`, the emulator-generated credential file for that port, the associated CA, and a bounded default-profile config whose endpoint/region/CA contents match. Ambient access keys, role/web-identity overrides, container credential providers, profile overrides and service-specific endpoint overrides are rejected. TLS verification remains enabled. Tests never initialize infrastructure or obtain live AWS credentials.

The expected snapshot requires:

- A bounded absolute JSON path and matching SHA-256.
- Pinned emulator version/commit provenance and the SHA-256 of the independently verified baseline summary.
- A summary whose declared check counts match its actual passing check list, including emulator identity, native terminal status, all native kinds, SQLite integrity, complete operation-attempt coverage, classified failures and registered-type reads.
- Exactly one nonempty case for each of the four scenarios, with resource/edge observation multisets, scope checkpoints, failures and expected terminal status.

Resources and edges are compared as multisets after removing **only** the top-level local `observed_at`. The removed timestamp is first checked against the real test observation interval. Nested property arrays keep their order; provider capture times, resource properties, tags, IDs, source attribution and failure messages remain exact. Checkpoint comparisons retain scope identity, last page, token presence, completion and failure flags. Actual opaque continuation-token bytes are not persisted in the oracle.

The test runs the public collector and drains its events into an in-memory SQLx Store. It checks Store-derived completion/partial status, then compares persisted resource observations, edge observations, checkpoints and failures. Differences show the collection, JSON path and hashes instead of dumping resource bodies. Channel capacity is 32; event count is capped at 2,000; cumulative event bytes are capped at 64 MiB; collection has a 180-second deadline. Timeout triggers cancellation and a bounded 15-second drain. Every test closes its Store explicitly, and the collector must leave no in-flight executor attempt.

## Preparing a mutation oracle

The Rust lifecycle wrapper starts and seeds the pinned local emulator, runs the ordinary CLI and independent fixture assertions, exports the snapshot, and then executes the Rust snapshot target twice without mutations. Only after both unchanged runs pass may it run the supplied mutation command. The recorder closes before those repeated scans; they use the same authenticated upstream loopback server directly to avoid exhausting the original evidence recorder's request cap.

```sh
cargo run -p inventory-harness -- mutate \
  --emulator /absolute/path/to/aws-emulator \
  --binary /absolute/path/to/unmodified/aws-inventory \
  --branch-probe /absolute/path/to/unmodified/emulator_branches \
  --output /absolute/path/to/new-oracle-evidence
```

Append `--` followed by the mutation gate's exact command to run it after the two clean baselines. Keep the server and command in this one wrapper lifetime. The wrapper can also provide a local FalkorDB when its explicit binary/module arguments are supplied. See [EMULATOR_TESTING.md](EMULATOR_TESTING.md) for the pinned installation and baseline limitations.

The first unchanged integration attempt correctly detected a moving Config `configuration_capture_time` for the synthetic SQS fixture. The retained evidence is `artifacts/emulator-oracle-v04-first/oracle-baseline-1.log`. This was fixture instability, not a killed mutant. The fixture lifecycle now stops its synthetic Config recorder only after all seeded records have appeared, checks two exact unchanged query results, and retains provider timestamps in the oracle. Any later change in a baseline must likewise be diagnosed before mutation results can be counted.

## Verification receipts and limits

The focused common-module run passed all 27 tests; its command/exit receipt is `artifacts/aws-properties-common-v04.json`, with detailed output in the corresponding `.log`. The complete 60-test AWS property/boundary suite then passed in 0.80 seconds of test execution after the origin fix; the command/exit receipt is `artifacts/aws-properties-retry-origin-v04.json`, and the detailed output is `artifacts/aws-properties-retry-origin-v04.log`.

After the 13 option regressions and finite-driver assertion repairs, the full AWS library suite passed **237 tests with zero failures or warnings**, including all 73 property/boundary/selector contracts. The test phase took 4.59 seconds and the recorded command took 47.833 seconds including compilation. The receipt and complete output are `artifacts/aws-survivor-tests-v04.json` and `artifacts/aws-survivor-tests-v04.log`. Those times describe this local verification run, not inventory throughput.

The four real collector cases also passed in each of two unchanged executions after the Config fixture was frozen. Those stability receipts are `artifacts/emulator-oracle-v04-second/oracle-baseline-1.json` and `oracle-baseline-2.json`, with corresponding logs. A mutation campaign establishes and records its own baselines against its frozen source and final ordinary binaries; these earlier development receipts do not replace that gate. Neither a compilation attempt nor a populated source file counts as a passing test.

The emulator tests prove behavior for generated fixtures and its supported protocol subset. They do not establish real AWS authorization isolation, all provider schema semantics, production service quotas, or a throughput limit. Property test case counts are not code coverage, and no target percentage should be inferred from them.

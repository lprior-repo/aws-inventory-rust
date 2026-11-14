# AWS SDK source and retry audit

> Status: Historical source-audit evidence. The typed-SDK-derived inventory
> and its reproduction crate were removed in M1.7; the typed service adapters
> and the independent conclusion of one retry owner remain. This document is
> not a current inventory, build recipe or claim about untested SDK releases.
> See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

This historical audit examined the 437 service SDK versions pinned at the time
to AWS's `release-2026-11-11` snapshot. Its archived machine report is
`artifacts/sdk-capabilities-v03.json`. It did not load credentials, construct
service clients, or make AWS requests.
The SDK already provides request classification, bounded retries, exponential backoff, jitter, retry budgets, and service-specific retry behavior. Those are preferable to duplicate application implementations when they can be integrated without breaking the application's quota and timeout contract. The inspected public interfaces do not provide a direct asynchronous admission point before both signing and attempt timeout. For the current strict contract, retaining one outer retry owner with SDK attempts set to one is the verified baseline; native retry migration remains conditional on an explicit timing-policy change or a suitable upstream hook. A retry budget is not a service's requests-per-second quota.

This document records SDK capabilities and integration constraints. It does not, by itself, assert that the inventory application has adopted every proposed integration change.

## Evidence and reproducibility

The completed source audit verified all **437** catalog archives, hashing **241,799,148 bytes**. It identified **19,617 operation modules**, **3,779 paginator modules**, and **407 waiter modules** after excluding generated matcher helpers. Every operation module registers the transient, modeled-retryable, AWS-error-code, and clock-skew classifiers. Every paginator exposes duplicate-token control. These counts describe generated source, not independently exercised operation behavior.

The audited generated sources total **647,023,212 bytes**. Native long-polling retry registration is present for SQS `ReceiveMessage`, Step Functions `GetActivityTask`, and SWF `PollForActivityTask` and `PollForDecisionTask`; the audit also verifies that their shared generated helper enables the long-polling retry-spec flag. The DynamoDB-specific four-attempt/25-ms configuration markers occur in DynamoDB and DynamoDB Streams.

All 437 configuration sources expose retry configuration, HTTP-client replacement, timeout configuration, retry partitions, interceptors, retry classifiers, behavior versions, async sleep, and shared endpoint-resolver replacement. None exposes a public runtime-plugin or retry-strategy setter. The report records declaration lines and source hashes for these findings.

The Rust auditor parses every local package manifest and checks its name and version against the catalog. It SHA-256 verifies every cached `.crate` archive against the catalog checksum. It reads each configuration source, operation index, operation module, paginator, and waiter module; checks that operation and waiter modules match their public index declarations; and produces a deterministic digest of the audited source files. Source fingerprints are recorded independently: the auditor does not unpack archives and byte-compare every local source file to its archived counterpart.

Public configuration methods are detected from their declaration lines. Classifier registrations and the long-polling retry marker are lexical source checks. The JSON records the exact patterns through the field names and the implementation, plus the auditor's own source hash. This is source evidence, not compilation or execution evidence for all service crates. No inference is made that an operation is safe to call, suitable for inventory, authorized by an account policy, available in every region, or implemented by this application.

The original source-audit executable and typed-SDK inventory were removed in
M1.7. Reproduction commands are intentionally omitted; the archived report and
verification notes are historical evidence only. This note does not provide a
current inventory of SDK releases or a way to regenerate the report.

Explicit bounds are 1,024 services, 16,384 entries per scanned directory, 100,000 operation modules and 100,000 waiter modules in total, 16 MiB per source/input file, 64 MiB per archive, 2 GiB of source reads, 1 GiB of archive reads, and 32 MiB of serialized output. Every source and archive read is capped even if a file grows after its initial size check. Processing is sequential; there are no spawned tasks or unbounded recursive directory walks.

### Completed verification

The default-feature audit example compiled and executed successfully without enabling service SDK features. Two executions of the final auditor produced byte-identical reports. The checked-in report is **2,304,697 bytes**, with SHA-256 `fb2cb8b410b151ba8a23a2e824fdf03b9d29d494809b181b4914d1d2d6c58ce9`; its embedded auditor-source hash also matches the checked-in Rust source. The catalog generator's `--check` completed successfully after its manifest template was updated to preserve the example's development dependencies.

The final locked Clippy check for the audit example completed with warnings denied as part of the successful version 0.3.0 verification script. The generated catalog check, application checks, tests and executable build also passed; see `VERIFICATION.md` and `artifacts/verification-v03.log`. Neither the audit nor its focused compilation enables `all-services`, and these checks do not establish runtime behavior of all SDK operations.

## Exact runtime versions inspected

| Crate | Pinned version |
|---|---|
| `aws-config` | `1.13.2` |
| `aws-runtime` | `1.10.4` |
| `aws-smithy-runtime` | `1.16.3` |
| `aws-smithy-runtime-api` | `1.21.0` |
| `aws-smithy-types` | `1.9.4` |
| `aws-smithy-http-client` | `1.5.2` |
| `aws-smithy-async` | `1.3.1` |
| `aws-sdk-dynamodb` | `1.139.0` |
| `aws-sdk-lambda` | `1.158.0` |
| `aws-sdk-ec2` | `1.278.0` |

These versions come from the local lockfile and downloaded source, rather than an assumption about the latest crate release. The source catalog records Smithy revision `ae943725d37495c5a6fdd8fc7112cd5e50f014fd` and AWS SDK revision `0f29bbb75b57d337bd00cf46c0d29d6988e2c372`.

## November 10, 2026 retry defaults

In the pinned runtime, `BehaviorVersion::latest()` is `v2026_11_10()`. The official Rust announcement says that this release makes Retry Behavior 2.1 the default for that behavior version and removes `AWS_NEW_RETRIES_2026`. The generic cross-SDK reference still includes earlier opt-in wording; the Rust-specific announcement and pinned implementation govern this project.

| Setting | General standard retries | DynamoDB and DynamoDB Streams |
|---|---|---|
| Attempts, including initial request | 3 | 4 |
| Transient base backoff | 50 ms | 25 ms |
| Throttling base backoff | 1 s | 1 s |
| Exponential backoff | Full jitter, bounded by configured maximum | Same |
| Default maximum exponential backoff | 20 s | Same |
| Retry budget | 500 tokens | Same |
| Transient/timeout retry cost | 14 tokens | Same |
| Throttling retry cost | 5 tokens | Same |

Explicit maximum-attempt settings override defaults. In particular, constructing `RetryConfig::standard()` explicitly supplies its own three-attempt value unless changed, including when used for DynamoDB. Native behavior includes modeled retry traits, standard AWS error codes, transient transport/timeout errors, HTTP status handling, and service clock-skew classification. It also refuses retries when the request body cannot be rewound.

The SDK's retry budget can stop retries before the configured attempt maximum. Its purpose is to reduce amplification during persistent failures; it is neither an in-flight semaphore nor a hard request-rate ceiling. Adaptive mode adds feedback-driven rate limiting, but it does not discover a service's actual control-plane quota. Its scope must match the affected throttling domain to avoid slowing unrelated requests.

### Server delay semantics are different from the earlier executor

`aws-runtime 1.10.4/src/retries/classifiers.rs` parses `x-amz-retry-after` as integer milliseconds. `StandardRetryStrategy::calculate_backoff` clamps it to the computed backoff through that backoff plus five seconds in Retry Behavior 2.1. Invalid values are ignored in favor of exponential backoff. This does not implement a strict bucket-wide cooldown for an arbitrary HTTP `Retry-After` delay.

Lambda's generated model exposes `TooManyRequestsException::retry_after_seconds()`. Its generated serde reads the `Retry-After` header and the modeled field, but the generic native classifier uses the AWS-specific millisecond header; merely having the typed field does not prove that native retries honor it. Removing custom strict cooldown behavior therefore changes that policy and must be documented explicitly. Native retry adoption should not be described as preserving the old fail-closed invalid/oversized-hint contract.

### Retry partition ownership

`RetryPartition::new(name)` uses runtime-global maps: equal names share a retry token bucket and adaptive limiter. The generated default is service-scoped. A quota policy scoped by account, region, and quota group should not accidentally use that broader scope for all failure feedback.

`RetryPartition::custom(name)` owns its state; cloning the partition shares it, while constructing another custom partition with the same name does not. This supports bounded caller-owned state. However, the pinned `TokenBucket::default()` and its builder defaults retain legacy costs. A custom partition that should preserve the new costs must receive an explicit bucket:

```rust
use aws_smithy_runtime::client::retries::{RetryPartition, TokenBucket};

let partition = RetryPartition::custom(bucket_key)
    .token_bucket(
        TokenBucket::builder()
            .capacity(500)
            .retry_cost(14)
            .timeout_retry_cost(14)
            .throttling_retry_cost(5)
            .build(),
    )
    .build();
```

This configures failure-retry budgeting. It does not replace the application's API request-rate policy.

## Public integration seams and their limits

| Seam | Pinned shape | Consequence |
|---|---|---|
| `HttpConnector` | `call(HttpRequest) -> HttpConnectorFuture` | Can await admission once per SDK transport attempt. |
| `HttpClient` | `http_connector(&HttpConnectorSettings, &RuntimeComponents) -> SharedHttpConnector` | Can wrap the selected pooled connector while forwarding timeout settings and validation/metadata behavior. |
| `Intercept` | Synchronous hooks returning `Result<(), BoxError>` | Can attach metadata or release attempt state; cannot await a semaphore or timer. |
| `RetryStrategy` | Synchronous decisions returning `ShouldAttempt` | Native orchestrator handles its requested delays, but normal generated clients expose no public setter for replacing the strategy. |
| `AsyncSleep` | `sleep(Duration) -> Sleep` | Has no operation or phase label; used by retry and timeout machinery. It is not a safe generic admission hook. |
| Shared `ResolveEndpoint` | `resolve_endpoint(&EndpointResolverParams) -> EndpointFuture` | Public asynchronous point before signing, but inside the attempt timeout; must delegate endpoint behavior and `finalize_params`. |

### Connector admission

The pinned orchestrator signs the request, runs before-transmit interceptors, constructs connector settings, calls `HttpConnector::call`, and awaits its response. It performs this sequence for each retry. This makes a connector wrapper a supported way to meter each SDK transmission attempt while leaving classification and backoff to the SDK.

Three contracts need explicit treatment:

1. **Signing age.** The request is already signed before a connector waits. Long queues, including a supported one-request-per-300-second policy, must not silently leave a signed request waiting for an arbitrarily long time.
2. **Timeout placement.** The operation-attempt timeout already surrounds the connector. Quota waiting can exhaust an attempt before any HTTP transmission. Native backoff itself is outside that attempt timeout and inside the overall operation timeout.
3. **Permit lifetime.** The connector returns an HTTP response whose body is consumed afterwards. Releasing an in-flight permit when the connector future completes bounds attempts through response headers, not full body consumption. Either a body/attempt lifetime guard or a separate documented bound on complete logical operations is needed for the stronger guarantee.

A wrapper should acquire no global permit during rate waiting, preserve the selected delegate's connection pooling and timeout configuration, and return/drop permits under cancellation without background tasks. Cancellation and a local policy rejection must not be converted into retryable I/O errors. `ConnectorError::user` is a non-retryable connector category in the inspected transient classifier; error reporting must retain the underlying local reason.

### Why the retry-strategy and sleep alternatives are not a drop-in solution

`try_op` repeatedly calls `should_attempt_initial_request`; `YesAfterDelay` makes it sleep and check again. Under Retry Behavior 2.1, the private `acquire_adaptive_send_token` helper also calls that method before every retry, before the attempt timeout and signing. Although named for adaptive mode, the inspected helper checks the retry-spec version rather than the retry mode.

That internal sequence would be suitable for a strategy wrapper delegating native retry decisions, but the generated `Config::Builder::runtime_plugin`, `push_runtime_plugin`, `CustomizableOperation::runtime_plugin`, and runtime-component fields are private to each SDK crate. A public trait in the runtime API is not evidence that a normal generated client lets an application install it. The audit separately records public method availability rather than counting private methods.

Using `AsyncSleep` to disguise a quota wait would mix admission with retry backoff and timeout clocks. It also has no initial request admission event when a native strategy returns `Yes` immediately. This does not preserve the quota contract.

The shared endpoint resolver can await admission before signing through the public `set_endpoint_resolver` method. It still runs inside the attempt timeout, so it solves the signing-age issue without automatically solving queue-time exclusion. It must preserve the generated resolver, endpoint parameters, authentication-dependent endpoint behavior, and parameter finalization.

## Recommended integration contract

There must be one retry owner per request. Under the current strict admission/timing contract, keep the outer owner and configure SDK maximum attempts to one. Do not enable native retries underneath that outer loop: the multiplication would bypass per-attempt admission and amplify failures. The all-SDK audit establishes available native facilities, not a reason to silently change those contracts.

If a later integration adopts native retry timing, make the SDK the sole retry owner and remove the corresponding outer loop. Retain only application-specific admission, cancellation, operation bounds, and reporting. Use standard retry mode as the default; adaptive mode should be an explicit choice for appropriately isolated traffic.

Choose timeout and permit semantics before migration. A supported endpoint/connector integration may use queue-inclusive SDK attempt timeouts, or separate admission waiting from an explicitly bounded transport/body lifetime. Neither is identical to the old executor simply by enabling SDK retries. An upstream public asynchronous pre-attempt admission hook would make the stronger integration smaller, but no such installable hook was verified in these generated clients.

Required focused integration checks are: a transient response followed by success uses exactly two admissions; denied/permanent failures remain single-attempt; no nested retry amplification; a fractional 300-second policy does not send early or produce a stale signature; unrelated buckets continue during waits; queue and transport timeouts have the documented boundaries; response-body ownership retains the intended permits; and cancellation releases every pending/in-flight lease. Synthetic SDK connectors can exercise these without live AWS credentials.

## Primary references and source locations

- [Official Rust retry announcement, updated for November 10](https://github.com/awslabs/aws-sdk-rust/discussions/1431).
- [AWS retry behavior reference](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html).
- [AWS Rust behavior versions](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/behavior-versions.html).
- [AWS Rust retry configuration](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/retries.html).
- Local `aws-smithy-runtime-api-1.21.0/src/client/{behavior_version,http,interceptors,retries,endpoint,runtime_plugin}.rs` defines the interfaces inspected here.
- Local `aws-smithy-runtime-1.16.3/src/client/orchestrator.rs`, `orchestrator/endpoints.rs`, `defaults.rs`, `retries.rs`, `retries/strategy/standard.rs`, and `retries/token_bucket.rs` establish ordering, timeout placement, and retry budget behavior.
- Local `aws-runtime-1.10.4/src/retries/classifiers.rs` establishes the AWS error-code and millisecond retry-header behavior.
- Local `aws-sdk-lambda-1.158.0/src/config.rs`, `client/customize.rs`, and `protocol_serde/shape_too_many_requests_exception.rs` establish generated-client visibility and typed delay deserialization.
- Local `aws-sdk-dynamodb-1.139.0/src/config.rs` establishes the service-specific 4-attempt/25-ms defaults.

The source audit and focused checks are not a full unsafe-code, dependency, cancellation, or performance certification of the SDK/runtime dependency tree. No all-service compilation or live AWS test is implied.

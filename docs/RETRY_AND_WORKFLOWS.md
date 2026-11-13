# Retry primitives and durable workflows

> Status: Kept and extended. The decision stands: one retry owner, a local
> durable queue, no workflow engine; the plan goes further by removing SDK
> retries altogether with the wire engine (ROADMAP M2) and taking retry classes
> from the catalog (M4.2). See [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

Research checked on **2026-11-12**. This is an architectural decision note, not
a claim that every integration below is installed or implemented. The current
scheduler contract is in [SCHEDULER.md](SCHEDULER.md); storage and delivery
guarantees are in [STORAGE.md](STORAGE.md) and
[ARCHITECTURE.md](ARCHITECTURE.md).

## Decision

Use the smallest mechanism that owns the failure being recovered:

| Failure boundary | Preferred mechanism | State that must survive |
|---|---|---|
| A transient AWS request failure | Current shared executor with SDK attempts set to one; reconsider native retries only with a verified quota boundary | Usually none beyond the running operation |
| Temporary graph connection failure | A bounded retry helper around one immutable, idempotent graph batch | The existing SQLx delivery acknowledgements |
| Scanner process exits between pages | Implemented SQLx local job queue; expired leases start fresh immutable scan attempts | Validated job intent, attempts, lease fencing and committed inventory evidence |
| Recovery across hosts/deployments | Optional Restate or Temporal integration, plus durable access to application data | Engine history and the inventory database or immutable page artifacts |

**Do not add a distributed workflow engine solely to retry AWS HTTP requests.**
The SDK already supplies service error classification, jittered backoff and a
retry token budget. A second general-purpose loop adds no value here. The
source audit found no public, asynchronous admission hook that preserves both
before-signing admission and the current queue-excluding attempt timeout.
Therefore this release retains its existing single retry owner, rather than
enabling hidden nested attempts. Standard retry mode does not provide
the scanner's account/region/service quota policy; adaptive mode shares feedback
across requests using the same retry partition and is not a general replacement for that policy.
[AWS retry behavior](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html)

For the remaining graph transport retry, **backon is the first library to
evaluate**: it decorates an async operation directly. Tower is appropriate when
the application already uses a `Service` middleware stack. Governor is useful
for rate admission, but it does not retry failed work. These are judgments about
the amount of integration this scanner needs, not benchmark claims.

If automatic recovery across deployments becomes a requirement, evaluate
**Restate first for a small independently operated durable service**. If an
organization already operates Temporal, use its Rust support rather than adding
a second engine. A **Temporal Standalone Activity** is sufficient for one durable
job; a Workflow is useful when coordinating several jobs and their outcomes.
The current Rust documentation explicitly supports Standalone Activities.
[Rust standalone quickstart](https://docs.temporal.io/develop/rust/activities/standalone-activities-quickstart)

Version 0.4 implements the local invocation-recovery boundary using SQLx and
SQLite; see [DURABLE_JOBS.md](DURABLE_JOBS.md). Its leased attempts are bounded,
and normal partial/failed scans are not automatically repeated. This does not
resume arbitrary AWS cursors or add a distributed quota coordinator.

## What each option actually adds

| Option | Application code and runtime | Persistence and scope | License |
|---|---|---|---|
| AWS SDK retries | Configure the clients already present; add the scanner-specific attempt admission boundary | In-process request recovery; no durable scan coordinator | Apache-2.0 for the AWS Rust SDK |
| backon 1.6.0 | Async closure plus retry builder and retryable-error predicate; `std` and `tokio-sleep` features suffice here | In-process; no database or server | Apache-2.0 |
| Tower 0.5.3 | `Service` adapter, `RetryLayer`, and a `Policy` implementing retry decisions and request cloning; optional timeout/concurrency layers | In-process middleware; no durable work queue | MIT |
| governor 0.10.4 | A quota and direct/keyed limiter; application still owns error handling, concurrent-work bounds and cancellation | Bundled state stores are in-memory; a custom store requires implementation | MIT |
| Restate Rust SDK 0.13.0 | Rust handler and HTTP endpoint, registered with a separate Restate server; server can begin as one self-contained binary | Durable invocation journal, timers, keyed objects and workflows | SDK MIT; server Business Source License 1.1 |
| Temporal Rust SDK | Rust Activity/Worker/client; add a Workflow for orchestration. Tokio and serialization are required | Temporal Cloud, or a self-hosted Temporal Service with persistence database | Rust SDK and server MIT |

Primary package/runtime evidence:
[AWS SDK license](https://github.com/awslabs/aws-sdk-rust/blob/main/LICENSE),
[backon manifest](https://docs.rs/crate/backon/1.6.0/source/Cargo.toml),
[Tower manifest](https://docs.rs/crate/tower/0.5.3/source/Cargo.toml),
[Tower retry policy](https://docs.rs/tower/0.5.3/tower/retry/trait.Policy.html),
[governor manifest](https://docs.rs/crate/governor/0.10.4/source/Cargo.toml),
[governor state and async API](https://docs.rs/governor/0.10.4/governor/struct.RateLimiter.html),
[Restate Rust SDK](https://docs.rs/restate-sdk/latest/restate_sdk/),
[Restate quickstart](https://docs.restate.dev/quickstart), and
[Temporal Rust quickstart](https://docs.temporal.io/develop/rust/quickstart).

Restate's server license explicitly permits internal deployments and production
deployments invoking the licensee's own services. Its restriction concerns a
public managed platform that lets third parties register and invoke their own
deployments directly through Restate APIs. Each release converts to Apache-2.0
after four years. The current server license is source available, rather than
an open source license. This distinction does not apply to the MIT Rust SDK.
[Server license](https://github.com/restatedev/restate/blob/main/LICENSE),
[SDK license](https://github.com/restatedev/sdk-rust/blob/main/LICENSE)

Temporal's Rust SDK reached **general availability on September 4, 2026**; it
should no longer be described as an experimental or preview Rust integration.
The official Rust quickstart uses the 1.0.0 crate family. Temporal's development
server can use SQLite, but official persistence documentation limits that
backend to development/testing. Production self-hosting adds a supported
persistence database; Temporal Cloud operates that layer for the user.
[GA announcement](https://temporal.io/changelog/rust-sdk-generally-available),
[persistence](https://docs.temporal.io/temporal-service/persistence),
[Rust SDK license](https://github.com/temporalio/sdk-rust/blob/main/LICENSE.txt),
[server license](https://github.com/temporalio/temporal/blob/main/LICENSE)

## Defaults and bounds that change the decision

| Mechanism | Documented default or behavior | Explicit setting needed here |
|---|---|---|
| AWS Rust SDK, November 10 behavior | General services: 3 total attempts, 50 ms transient base; DynamoDB/Streams: 4 attempts, 25 ms transient base. Throttling base remains 1 second. | Pin the behavior version, total attempts, per-attempt timeout and total operation timeout. |
| backon exponential builder | 1 second initial delay, factor 2, 60 second base cap, `max_times = 3`, jitter disabled | Set the count, enable jitter deliberately, classify errors, and bound both each operation and total elapsed time. |
| Tower retry | Application-defined `Policy`; no universal count/error policy follows merely from adding the layer | Implement a finite policy and put timeout/admission layers at the intended scope. |
| governor | Quota constructors also select burst capacity; `per_minute(60)` permits a burst of 60 | Use an explicit interval/burst policy. Bound the key registry and in-flight work separately. |
| Restate invocation | Current server docs: 50 ms initial interval, factor 2, 60 second maximum interval, 70 attempts then pause | Set invocation and run-step policies explicitly, including what exhaustion means. |
| Temporal Activity | 1 second initial interval, factor 2, maximum interval 100 times the initial interval; unlimited attempts by default | Set total attempts, Start-to-Close and Schedule-to-Close limits, and permanent-error handling. |

Sources:
[AWS November update](https://github.com/awslabs/aws-sdk-rust/discussions/1431),
[backon defaults](https://docs.rs/backon/1.6.0/backon/struct.ExponentialBuilder.html),
[Tower policy](https://docs.rs/tower/0.5.3/tower/retry/trait.Policy.html),
[governor quota](https://docs.rs/governor/0.10.4/governor/struct.Quota.html),
[Restate service configuration](https://docs.restate.dev/services/configuration.md),
[Temporal retry policy](https://docs.temporal.io/encyclopedia/retry-policies).

Several details matter when translating a configuration:

- **AWS defaults are versioned.** `aws-config` 1.13.2 with
  `BehaviorVersion::v2026_11_10()` or later enables the new behavior and removes
  the earlier `AWS_NEW_RETRIES_2026` flag. An explicit `RetryConfig` overrides
  maximum attempts, including the service-specific DynamoDB default. The dated
  Rust announcement resolves older generic documentation that still describes
  an opt-in flag.
  [AWS announcement](https://github.com/awslabs/aws-sdk-rust/discussions/1431)
- **Backon counts generated retry delays.** Its default three delays can produce
  four operation calls: the first call and three retries. Jitter adds a random
  amount below the current base delay, so a configured base cap is not a strict
  cap on the final jittered sleep. `with_total_delay` bounds cumulative sleep,
  not time spent executing operations.
  [Retry state machine](https://docs.rs/backon/1.6.0/src/backon/retry.rs.html),
  [exponential implementation](https://docs.rs/backon/1.6.0/src/backon/backoff/exponential.rs.html)
- **Restate has separate invocation and run-step policies.** The Rust overview
  still uses indefinite-retry wording, while current server configuration
  documents a finite default followed by suspension. Unset Rust options use
  server/service settings. `RunRetryPolicy` also warns that actual attempts and
  elapsed time can exceed its configured values when execution finishes before
  its result reaches Restate. Treat these as orchestration bounds, not an exact
  wire-attempt quota.
  [Rust configuration](https://docs.rs/restate-sdk/latest/restate_sdk/configuration/index.html),
  [run policy](https://docs.rs/restate-sdk/latest/restate_sdk/context/struct.RunRetryPolicy.html),
  [current error-handling guide](https://docs.restate.dev/guides/error-handling.md)
- **Temporal Activity retry and Workflow retry are distinct.** Activities have
  retries by default; whole Workflow Executions do not. Workflow replay restores
  execution state and does not mean starting the whole scan again.
  [Temporal retry policy](https://docs.temporal.io/encyclopedia/retry-policies)

## Minimal integration path for this scanner

### 1. Give AWS request retries one owner

SDK standard retry with a small application-specific admission component is a
candidate future simplification, conditional on a usable public attempt hook.
The exhaustive [SDK source audit](SDK_RETRY_AUDIT.md) does not establish that hook
in the pinned release. Keep the outer executor as the sole retry owner now.
Only remove its AWS retry/backoff loop after a replacement's admission, signing,
timeout, cooldown, and cancellation behavior has been demonstrated. Do not stack
a backon loop around an SDK operation that already retries.

The public Smithy `HttpClient`/`HttpConnector` interfaces provide an async
transport boundary, unlike synchronous interceptor hooks. A wrapper can delegate
to the normal pooled HTTP implementation and meter each transport attempt. It
must preserve connector settings and runtime components. This is an integration
candidate, not proof that arbitrary waiting at that boundary is safe.
[Pinned Smithy HTTP interfaces](https://github.com/smithy-lang/smithy-rs/blob/ae943725d37495c5a6fdd8fc7112cd5e50f014fd/rust-runtime/aws-smithy-runtime-api/src/client/http.rs)

Resolve these concrete details before switching the retry owner:

1. **Signing and waiting:** connector admission occurs after request signing.
   A long quota wait can leave a signed request stale. Prefer a supported
   pre-signing attempt boundary when available; do not hide scheduling inside
   an unrelated SDK sleep implementation.
2. **Timeout scope:** SDK attempt time includes work before and during transport.
   A quota wait must not consume every retry without sending a request. Separate
   queue/admission deadlines from transport deadlines, or explicitly document
   the combined budget and test slow quotas.
3. **Permit lifetime:** connector completion can precede response-body reading.
   Decide whether the semaphore limits connection/header work or complete
   response processing. Retain a separate bound for active logical operations
   and body memory if the connector permit ends earlier.
4. **Feedback:** SDK-native server-delay handling is not automatically equivalent
   to a scanner-wide cooldown for every caller in the same quota bucket. Keep
   only explicitly required shared policy; document any semantic change rather
   than retaining a second retry engine for compatibility by accident.

The v0.2 implementation uses one SDK attempt and application-owned retries. That
is a description of the baseline, not a recommendation to preserve duplicated
retry machinery. If the public SDK boundary cannot satisfy the required policy
with a small reviewed wrapper, record that limitation and choose the smallest
correct alternative explicitly.

Governor can replace fixed-rate timing machinery if its quota semantics fit the
chosen contract. `Quota::with_period(interval)` expresses one admission per
interval; per-second/minute constructors may allow larger bursts. The built-in
keyed store needs housekeeping, and it does not supply our global/per-bucket
concurrency limits or shared server cooldown. Adding governor is useful only
if it actually removes code after those requirements are settled.
[Quota semantics](https://docs.rs/governor/0.10.4/governor/struct.Quota.html),
[keyed-state housekeeping](https://docs.rs/governor/0.10.4/governor/struct.RateLimiter.html)

### 2. Retry one immutable graph batch

The existing graph plan and acknowledgement log already supply a useful retry
boundary: execute the same idempotent batch, then record its acknowledgement in
SQLx. A lost response after a server commit may cause replay. It must replay
identical `MERGE` identities, not append another observation or create a new
scan. See [graph delivery](ARCHITECTURE.md#9-falkordb-projection).

A minimal backon integration would decorate a function that reconnects when
needed and sends exactly one batch. The following illustrates the library API;
the application-specific functions and types are not an installed adapter:

```rust,ignore
use backon::{ExponentialBuilder, Retryable};
use std::time::Duration;
use tokio::time::timeout;

let retry = (|| send_same_batch_with_reconnect(&destination, &batch))
    .retry(
        ExponentialBuilder::default()
            .with_min_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(2))
            .with_jitter()
            .with_max_times(4) // Initial call plus at most four retries.
            .with_total_delay(Some(Duration::from_secs(8))),
    )
    .when(is_transient_transport_error);

let acknowledgement = timeout(Duration::from_secs(30), retry).await??;
persist_acknowledgement(&batch, acknowledgement).await?;
```

Keep the existing per-send timeout as well. Retry connection loss and ambiguous
transport timeouts; preserve authentication, malformed Cypher, changed batch
digest and local validation failures as terminal errors. Cancellation should
drop the owned future, with no detached retry task. The outer deadline bounds
elapsed time; the base-delay cap of two seconds allows jittered sleeps below
four seconds. On exhaustion, return the failure and retain SQLx progress for a
later invocation. A graph reset requires a fresh delivery checkpoint.

### 3. Add durable orchestration only for process recovery

An engine earns its runtime cost when a job must outlive the initiating CLI,
resume after deployment, expose operator controls, or coordinate workers.
The smallest useful first job here is **delivery of an immutable graph plan**:
it already has stable input and replay-safe effects. Wrap that job in a Restate
handler or a Temporal Standalone Activity, with a stable job ID, finite retry
policy and bounded execution time. Continue using SQLx delivery acknowledgements.

For Restate, a service struct holds clients/configuration; a macro defines a
handler; `ctx.run` journals an external operation's result. Expose the service
through the SDK HTTP endpoint and register it with Restate. Use `TerminalError`
for permanent failures. Set an ingress idempotency key yourself when separate
client submissions should deduplicate: automatically assigned ingress keys do
not deduplicate distinct submissions. The default idempotency retention is
24 hours; persistent application acknowledgements have a different lifetime.
[Rust handler model](https://docs.rs/restate-sdk/latest/restate_sdk/),
[journaled operations](https://docs.rs/restate-sdk/latest/restate_sdk/context/trait.ContextSideEffects.html),
[ingress idempotency](https://docs.restate.dev/services/invocation/http.md)

For Temporal, register an Activity on a Worker and start it using a client,
Task Queue, Activity ID and close timeout. Add a Workflow only when there is
actual multi-step orchestration. A scan coordinator could schedule bounded
account/region/service shards and aggregate success, partial failure and
cancellation. Worker restarts recover through Temporal; external effects still
need idempotency, because a worker can finish a write and fail before reporting
completion. Bound SDK attempts inside each Activity and durable Activity attempts
separately: four SDK attempts inside three Activity attempts can cause twelve
transport attempts for one repeated logical request.
[Rust standalone API](https://docs.temporal.io/develop/rust/activities/standalone-activities-quickstart),
[Activity idempotency](https://docs.temporal.io/activity-definition)

## Data correctness that a workflow engine does not supply

The storage transaction atomically commits observations, edges and a page
checkpoint. Exact replay is accepted, but different content at the same
`(scan_id, scope_key, page_number)` is a conflict. Refetching an AWS page can
change data and observation timestamps. Therefore, wrapping the existing whole
scan function in an automatic retry does not implement correct resume.
[Page transaction and replay contract](STORAGE.md#atomic-pages-and-replay)

Before durable scan orchestration, choose a recoverable page protocol:

1. Fetch one bounded page and durably retain its immutable normalized envelope
   and observation timestamp, or an immutable artifact reference plus digest.
2. Apply that envelope through the existing atomic store operation. If the
   commit acknowledgement is lost, replay the retained envelope exactly.
3. Advance orchestration from the committed continuation token. Retain child
   scopes and their failures; a denied or incomplete scope remains partial.

Restate can journal the fetch result before a separate idempotent store step.
Temporal can use separate Activities or a durable artifact/receipt. Large page
bodies should not be assumed to fit an engine's history limits; bound payloads
and use durable artifact references when needed. Keep the AWS pagination and
fan-out limits regardless of engine choice. No workflow journal turns an
eventually consistent multi-service inventory into a point-in-time snapshot.

Finally, a local SQLite file is not automatically accessible to a replacement
worker on another machine. The first integration must keep a single writer and
durable volume accessible to its worker, or introduce a storage service/shared
durable artifact design. Restate or Temporal owns orchestration durability;
SQLx owns inventory commits. Their journals are separate transactions. The
idempotent page and graph protocols reconcile that boundary.

## Acceptance criteria for the chosen implementation

Prove the actual failure boundary with focused tests: every SDK retry is admitted;
slow quota waits do not send stale signed requests; cancellation releases permits;
permanent errors stop; retry exhaustion stays bounded; graph commit followed by
lost acknowledgement replays without extra nodes or edges. For a future durable
scan coordinator, also crash between fetch, journal, database commit and receipt,
then verify exact page recovery and truthful partial coverage.

This research did not run a workflow server, alter Cargo dependencies, invoke a
live AWS account, or measure workflow throughput. The recommendation is based on
the linked primary interfaces, defaults and the scanner's existing persistence
contract. Integration tests and the delivered source determine what is actually
supported in a release.

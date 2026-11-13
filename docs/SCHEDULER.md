# API admission, pacing, retries, and cancellation

## Contract and scope

`inventory_engine::limiter::ApiExecutor` owns the process-local request budget.
Every explicit inventory API attempt, including a retry, must enter this executor. AWS SDK clients
must use a maximum of one SDK attempt so that an inner SDK retry cannot bypass
the executor's accounting. The operation supplied to `execute` must be a
repeatable, cancellation-safe inventory read.

Credential-provider HTTP work, such as SSO refresh, is outside this inventory
request budget. The executor meters SDK operation admission. The SDK may still
wait for credentials or a connection before transmitting a request; network
delivery timing is not controlled by this gate.

The executor does not create worker tasks or buffer API response bodies. Its
caller owns the operation futures, bounds collection fan-out, and drains those
futures during shutdown. The AWS adapter owns page limits, token-cycle detection,
response mapping, and payload limits. A successful request is not proof that an
entire service or account was collected.

## Quota keys

A bucket identifies a **partition, account, region, service, and quota group**.
Calls sharing an AWS throttle budget must share a bucket even when their
operation names differ. A quota group may also contain a single API operation.
Global services need a consistent global scope rather than a separate budget
for every discovery region.

The native adapter builds keys in this exact format:

`partition:account:region:service:quota-group`

For example, its current EC2 reads use
`aws:123456789012:us-east-1:ec2:read`, and Lambda control reads use
`aws:123456789012:us-east-1:lambda:control_read`. The EC2 group deliberately
combines several independently limited actions; this conservatively shares
capacity rather than claiming those actions share one AWS quota. The bootstrap
identity request uses `identity:REGION:sts` before the account ID is known.

Bucket strings are caller-defined. The executor cannot discover AWS quota
relationships, infer cross-region quotas, or coordinate a budget with another
process. Creating a separate key for every request would defeat rate limiting.
The configured RPS is an upper bound for each bucket, not a global RPS limit.

## Configuration and hard bounds

The table lists `ExecutorConfig` library defaults. The CLI selects 128 global
reservations and four per bucket by default; its rate and retry defaults match
the library. Both entry points enforce the same hard bounds.

| Setting | Default | Accepted range or relationship |
|---|---:|---|
| `global_concurrency` | 64 | 1–4,096 |
| `per_bucket_concurrency` | 8 | 1–`global_concurrency` |
| `requests_per_second` | 5 | 1–100,000; default for buckets without an override |
| `max_attempts` | 5 | 1–10, including the initial attempt |
| `attempt_timeout_ms` | 30,000 | 1–300,000 |
| `max_backoff_ms` | 30,000 | 1–300,000 |
| Period-policy `requests` | — | 1–100,000 |
| Period-policy `period_ms` | — | 1–300,000; resulting rate must not exceed 100,000/s |
| Server retry delay | — | 0–300,000 ms; larger directives close the bucket |
| Retry-hint input | — | At most 128 bytes per value and eight values per supported header |
| Registered buckets | — | At most 65,536 for an executor lifetime |
| Exact bucket policies | — | At most 65,536 at construction |
| Pending operation futures | — | At most 65,536 admitted to an executor |
| Bucket key length | — | 1–512 UTF-8 bytes |
| Retained retry message | — | At most 1,024 Unicode characters plus category/directive labels |

Validation happens in every constructor. Invalid configuration is a typed
`InvalidConfig` error. Pending admission and registry exhaustion are explicit
`Exhausted` errors. There is no unbounded background queue. Registry entries
remain until the executor is dropped; no eviction can accidentally reset a
recent bucket's rate history.

These limits are implementation ceilings, not proposed AWS quotas. Exact period
policies support rates down to **one request per 300 seconds**, or **1/300 RPS**.
Slower quotas need an outer schedule or an explicitly extended bound. Decimal
floating-point rates are not accepted; a request count and period represent the
desired rational ceiling without binary floating-point conversion.

## Explicit limits for individual buckets

The existing `ApiExecutor::with_bucket_policies(config, policies)` accepts a
`BTreeMap<String, BucketLimits>`. Each `BucketLimits` has
`requests_per_second: u32` and `concurrency: usize`. Both are required;
deserialization rejects unknown fields. The rate must be within 1–100,000 and
the concurrency within 1–`global_concurrency`. Its Rust API and JSON format
remain supported.

`ApiExecutor::with_rate_policies(config, policies)` accepts a
`BTreeMap<String, BucketPolicy>`. A `BucketPolicy` is either
`PerSecond(BucketLimits)` or `PerPeriod(PeriodLimits)`, with the latter containing
`requests: u32`, `period_ms: u64`, and `concurrency: usize`. The JSON object uses
either `requests_per_second` or the pair `requests`/`period_ms`. Both shapes
reject unknown fields, and mixed shapes fail deserialization. Configuration is
validated and converted into bounded scheduler state before requests run.

The scan CLI loads `BucketPolicy` objects through `--rate-policy FILE`, with a 1 MiB input
limit and at most 4,096 entries. The engine constructor independently enforces
its own hard bounds even when used without the CLI.

Matching uses the entire key, including account and region. There are no glob,
prefix, suffix, or wildcard rules. Unmatched keys use `ExecutorConfig` defaults.
The policy map is fixed when the executor is created. Every bucket retains its
own concurrency limit and adaptive rate ceiling for the executor lifetime;
changing a file during a scan does not reset pacing or alter an active bucket.

An illustrative JSON policy for the two current native groups is:

```json
{
  "aws:123456789012:us-east-1:ec2:read": {
    "requests_per_second": 8,
    "concurrency": 4
  },
  "aws:123456789012:us-east-1:lambda:control_read": {
    "requests": 1,
    "period_ms": 2000,
    "concurrency": 2
  }
}
```

The Lambda example deliberately admits one attempt every two seconds. Both
values allocate a scan budget with headroom; they do not claim to be the
maximum throughput available to the account. Other applications, consoles, and
scan processes consume AWS capacity too. The executor cannot observe their
traffic or coordinate with them.

### Choosing a ceiling from AWS documentation

The relevant AWS limits vary by operation and request shape. The following
official defaults were checked on 2026-11-12:

| AWS operation or group | Sustained request allowance | Implication for this adapter |
|---|---:|---|
| EC2 ordinary non-mutating APIs | 20/s per API action | The combined `ec2:read` key uses a smaller shared budget. |
| EC2 listed describe APIs without filtering or pagination | 10/s per API action | Unfiltered, unpaginated requests may have lower limits. |
| EC2 paginated `DescribeInstances` and `DescribeVolumes` | 100/s per API action | A higher per-action ceiling needs more precise bucket grouping. |
| Lambda control APIs excluding invocation, `GetFunction`, and `GetPolicy` | 15/s shared across the group | `ListFunctions` belongs in the shared `lambda:control_read` bucket. |
| Lambda `GetFunction` / `GetPolicy` | 100/s / 15/s respectively | Future collectors should use their separate quota groups. |

Sources: [EC2 request throttling](https://docs.aws.amazon.com/ec2/latest/devguide/ec2-api-throttling.html)
and [Lambda quotas](https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-limits.html).
These are documented defaults, not a live lookup of the user's account quotas.
Burst capacities are deliberately excluded from this smooth pacing model.

For throughput close to a particular API limit, confirm its request shape,
quota scope, current account allowance, and headroom for other callers. Refine
collector bucket groups where independent actions are currently combined, then
apply exact policies. AIMD responds to observed throttles but cannot establish
an undocumented quota or guarantee that no throttle occurs. Automatic Service
Quotas discovery is not implemented.

## Admission order

An attempt follows this sequence:

| Step | Reservation or action | Why it is ordered here |
|---|---|---|
| 1 | Acquire the bucket's concurrency reservation | Bounds work competing inside one quota family. |
| 2 | Acquire the bucket's dispatch reservation | Allows only its next sender to queue for global capacity. |
| 3 | Wait until both pacing and the shared server cooldown permit a start | Uses no global concurrency capacity while waiting. |
| 4 | Acquire global concurrency capacity | Only a locally eligible sender enters the global queue. |
| 5 | Atomically recheck eligibility and, if eligible, commit the admission timestamp | A preceding request may have returned a throttle or retry delay while this sender waited. |
| 6 | If eligibility moved forward, release global capacity and return to step 3 | A throttled family cannot occupy global slots while sleeping. |
| 7 | Release the dispatch reservation | The next sender measures spacing from the committed SDK-operation admission. |
| 8 | Run the operation under the attempt timeout | Holds global and local in-flight reservations only while executing. |
| 9 | Apply rate feedback and server directives, then release both reservations | Feedback is visible before capacity is handed to another request. |

The dispatch reservation protects ordering, not a mutable data borrow. State
mutexes are held only in synchronous helpers and never across `.await`. Final
eligibility checking and timestamp publication share one short critical section;
a concurrently reported cooldown cannot slip between those two transitions. The
executor uses Tokio's fair semaphores for reservations and has no detached
tasks, manual polling loops, or per-attempt task spawn.

### Why timestamps are committed at dispatch

Reserving future timestamps before waiting for global capacity is incorrect:
several expired reservations can become runnable at once when global capacity
returns. This creates a burst despite apparently valid reservations. Here,
`last_start` advances only after global capacity is available and the final
eligibility check passes. A same-bucket successor must then wait a full interval
from that actual timestamp.

For an integer policy, the interval is `ceil(1,000,000,000 / effective_RPS)`
nanoseconds. A period policy starts at
`ceil(period_ms × 1,000,000 / requests)` nanoseconds. For example, three requests
per ten seconds produces a 3,333,333,334 ns minimum interval. Integer arithmetic
rounds conservatively upward by less than one nanosecond; no floating-point
accumulator or future rate tickets are used. No token-bucket burst allowance is provided. Timer resolution
and API latency can reduce achieved throughput; the configured rate is a cap,
not a throughput promise.

### Bound on admission rechecks

While one sender owns the dispatch reservation, no additional request from that
bucket can start. The number of active attempts that can publish a new rate or
cooldown is
at most the bucket's concurrency limit minus one. Feedback occurs before their local reservation is
released. Each deferred recheck therefore corresponds to one of a bounded set
of old attempts. The implementation also enforces a static ceiling of 4,097
admission rounds and returns a typed failure if it is reached.

## Retry classification and backoff

`Throttled`, `Transient`, and `Timeout` are retryable. `RetryAfter` carries the
same classification through `RetryKind`, a bounded message, and a `RetryAfter`
directive. `Denied`, `Permanent`, `InvalidConfig`, `Cancelled`,
`RetryAfterRejected`, and admission failures are terminal. A terminal
denial is preserved as a coverage failure; it must not be reported as an empty
resource inventory.

The retry loop has a static ceiling of ten attempts, further restricted by the
validated configuration. Exhausting a retry budget returns `Exhausted` with the
last retryable error category and bounded detail. Attempt timeouts are counted
individually even when the final result is retry exhaustion.

After retryable failure number `n`, counting from zero, the backoff ceiling is:

`min(max_backoff_ms, 100 ms × 2^n)`

The actual backoff uses full jitter between zero and that ceiling, inclusive.
The arithmetic is checked, saturating, or deliberately wrapping for the jitter
permutation. No concurrency reservation is held during backoff. A retry must
re-enter the same pacing and concurrency admission path as the initial attempt.

`new` seeds jitter once from the standard library's randomized hash state.
`with_seed` accepts an explicit seed for deterministic replay tests. The jitter
generator is for load spreading, not cryptography.

## Server-directed cooldowns

The AWS adapter preserves these response hints:

| Source | Interpretation |
|---|---|
| HTTP `Retry-After` | Unsigned integer seconds, or an IMF-fixdate HTTP date parsed by Smithy |
| AWS `x-amz-retry-after` | Unsigned integer milliseconds |
| Lambda `TooManyRequestsException.retry_after_seconds()` | Modeled integer-seconds string, including when no raw header is available |

The adapter considers every value up to the header-count bound and selects the
longest valid delay across sources. Invalid UTF-8, malformed values, excessive
numbers, and oversized/count-exceeding headers remain explicit directives;
they are not silently discarded. Standard header dates use an injected wall
clock in parser tests and the local system clock in production. Smithy's parser
accepts IMF-fixdate; obsolete HTTP-date forms are rejected. Absolute-date
interpretation depends on that wall clock being correct. Past dates mean zero
additional delay. Integer hints and the scheduler itself use no wall-clock
arithmetic after parsing.

When an attempt returns a valid hint, the executor sets:

`cooldown_until = max(existing_cooldown_until, observed_failure_time + delay)`

The observation time is a Tokio monotonic instant taken when the executor
processes the failure. This deadline applies to **every new attempt in that
bucket**, including peers and retries, and is published even when the reporting
request has exhausted its own attempts. Successes and shorter later hints do
not clear or shorten it. Already admitted operations can finish while a valid
cooldown is pending. Queued senders recheck the deadline after gaining global
capacity and release that capacity before waiting again.

A server delay can exceed `max_backoff_ms`; it is honored independently up to
the 300,000 ms server-delay bound. Ordinary jitter still applies, and whichever
of pacing, jitter, and the shared deadline finishes last governs the next
admission. No global permit is held during these waits.

An invalid hint, a delay above 300,000 ms, or an unrepresentable monotonic
deadline produces `RetryAfterRejected`. The original request retains its error
category and bounded detail in that rejection. The bucket becomes terminal for
the executor lifetime. A bucket-local cancellation signal immediately wakes
local, dispatch, pacing, global-capacity, and backoff waiters and drops active
same-bucket SDK futures; these calls report the explicit bucket rejection.
Other buckets remain usable. Recreating an executor is an explicit caller
decision; there is no automatic reset that could bypass the rejected directive.
Permission and permanent errors remain terminal even if their responses contain
a retry header.

Header semantics are sourced from [RFC 9110 section 10.2.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.3),
the [AWS retry-behavior reference](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html),
and [Lambda ListFunctions errors](https://docs.aws.amazon.com/lambda/latest/api/API_ListFunctions.html).
The typed Lambda API was checked against
[aws-sdk-lambda 1.158.0](https://docs.rs/aws-sdk-lambda/latest/aws_sdk_lambda/types/error/struct.TooManyRequestsException.html)
and the fetched pinned source. This executor owns its cooldown policy; the
SDK's built-in retries remain disabled.

## Adaptive rate feedback

A throttle on an integer policy retains the previous behavior: halve the
effective integer RPS with a floor of one; each success adds one up to the
configured ceiling.

A period policy doubles its current nanosecond interval on a throttle, with a
maximum interval of 300 seconds. Its minimum effective rate is therefore 1/300
RPS, including at the slowest supported configured ceiling. Each success adds
one to a recovery counter capped at 100,000. Starting from the interval recorded
at the last throttle, the effective interval is
`ceil(1e9 × base_interval_ns / (1e9 + recovery_count × base_interval_ns))`,
bounded below by the original policy interval. A new throttle doubles the
current effective interval and resets the counter. Retaining these integer
credits prevents nanosecond rounding from discarding every small recovery
increment at high rates. The calculation uses bounded integer arithmetic and
conservative rounding. Sub-one-RPS policies can therefore slow down and recover
without crossing their fractional ceiling.

Transient transport failures do not alter the rate. This is
multiplicative decrease with additive recovery, applied only to the bucket
that received the response.

The next sender rechecks feedback after waiting for global capacity. Unrelated
buckets keep their own pacing state and can use the global capacity released
by a throttled bucket. This is not distributed quota coordination, dynamic
Service Quotas discovery, or an implementation of every AWS service's throttle
model.

## Timeout, cancellation, and ownership

The attempt timeout begins after admission. Admission, cooldown, and backoff waits remain
cancellable but do not consume the attempt timeout. An overall scan deadline
belongs to the caller. The operation callback and future must not block a
runtime thread: Tokio's cooperative timeouts cannot preempt a blocking poll.

`cancel()` permanently cancels an executor. `is_cancelled()` allows supervisors
to skip queued work before starting configuration or provider loading, and
`cancelled().await` observes the same signal during caller-owned asynchronous
waits. Both are observation-only; neither can reset the executor.

Cancellation wakes local admission,
dispatch, rate/cooldown, global admission, backoff, and active attempt waits. Dropping an
`execute` future also drops its operation future and releases every owned
reservation. The request record counts that abandonment as cancellation.

No background work is created by this module, so there are no executor-owned
join handles to leak. The scan supervisor still must stop new intake, call
`cancel`, poll/drain its owned futures, finalize already committed storage work,
and report a cancelled or partial scan. Dropping an HTTP future does not promise
that AWS stopped processing a request already transmitted; inventory reads are
therefore required to be repeatable and free of mutation.

## Observability and evidence

`ExecutorStats` provides attempts, retries, throttles, timeouts, accepted server
delay hints (`server_delays`), rejected hints (`rejected_retry_hints`), completed
successes, terminal failures, cancellations, current/peak in-flight attempts,
and registered bucket count. Success/failure/cancellation count logical
operations; attempts/retries/timeouts count individual attempts. Hint counters
count reporting attempts, including a valid zero delay or a final failed
attempt. Peers awakened by the same rejection do not increment the rejected-hint
counter again. Concurrent
snapshots are approximate because counters are independent atomics. A snapshot
after the caller has drained its futures is final for that executor.

The async shell has tracing spans. Counters saturate rather than wrap; only the
jitter nonce and permutation deliberately wrap. Common successful attempts do
not allocate error strings or retain response copies. Bucket registration
allocates bounded registry/key/state storage, and the Tokio runtime may allocate
its own synchronization and timer bookkeeping. No zero-allocation or fastest
implementation claim is made without an allocation/profile benchmark.

The deterministic integration suite is `crates/engine/tests/limiter.rs`:

```bash
cargo test -p inventory-engine --test limiter --locked
```

Observed on 2026-11-12: **42 passed, zero failed**. It covers rate limits on initial calls and retries, fractional interval
rounding, global-contention burst prevention, local/global caps, unrelated
bucket progress, throttle feedback during global waits, adaptive recovery,
retry classification, exact attempt ceilings, timeouts, every cancellation
wait category, dropped-future cleanup, hard configuration/key/policy bounds,
exact policy matching, overridden rates and concurrency, policy-specific
adaptive ceilings, and bounded reproducible jitter. The new cases cover exact
period-policy decoding/bounds, sub-one-RPS pacing, fractional recovery, final
attempt hints affecting peers, successive deadline extension/non-shortening,
global-capacity release and rechecking, cooldown cancellation, all rejected-hint
wakeup paths, excessive/invalid hints, and the accepted 300-second boundary.
Tests use paused Tokio time
and own their futures
directly; they do not make AWS calls.

A pure rate-state regression in `crates/engine/src/limiter.rs` separately checks
that accumulated recovery credits make progress from 50,000 toward 100,000 RPS,
never cross the configured ceiling, and reset on the next throttle. This
arithmetic check is independent of Tokio's coarser timer resolution.
Observed after that correction: `cargo test -p inventory-engine --lib --test
limiter --locked` passed all three engine unit tests (including the new rate
regression) and all 42 limiter integration tests. Strict engine source Clippy
also passed with warnings denied and explicit unused-async, lock-across-await,
large-future, string-slice, get-unwrap, and ignored-must-use checks.

AWS adapter parser/classifier tests are in `common.rs` and `lambda.rs`. They use
synthetic SDK responses and cover distinct second/millisecond units, HTTP dates,
invalid/oversized/count-exceeding values, longest-hint selection, typed Lambda
metadata, and terminal authorization errors. Their execution is tracked in the
consolidated native-adapter validation; the engine result above does not claim
that those SDK tests or a live AWS scan ran.

This change makes no throughput or allocation-performance claim. Full skill
verification remains subject to the missing tools and reference bundle recorded
in [RUST_REVIEW.md](RUST_REVIEW.md); the async skill's referenced
`references/cancellation-safety.md` was again unavailable during this extension.
The Holzman references used here were `nasa-jpl-standards.md`,
`runtime-performance-architecture.md`, `zero-cost-abstractions.md`, and
`latency-throughput-playbook.md`.

## AWS SDK references

- [AWS SDK for Rust retry configuration](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/retries.html)
- [AWS SDK for Rust timeout configuration](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/timeouts.html)
- [AWS SDK for Rust concurrency](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/concurrency.html)
- [AWS SDK for Rust crate fundamentals](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/fundamentals.html)

The scheduler source is authoritative for the implementation described here.
AWS references describe the surrounding SDK behavior and do not certify this
executor's correctness or universal quota coverage.

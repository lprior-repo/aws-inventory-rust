# Continuous AWS CMDB: performance contract proposal

Status: DRAFT engineering proposal, 2026-11-13. The user requires thousands of
AWS accounts, continuous inventory, a performant GPU-rendered UI, and replacement
of their Steampipe/CloudQuery use. This document proposes acceptance criteria;
it does not establish deployed fleet capacity or competitive superiority.

The subsequent full [CMDB specification draft](cmdb-spec/architecture-spec.draft.md)
records the user's confirmed commercial-only, self-hosted, hourly-reconciliation
scope. Its explicit decisions supersede the earlier candidate budgets here;
final policy/pre-mortem choices remain visible in that bundle.

## What we are competing with

| System | Documented execution model | Design implication |
| --- | --- | --- |
| Steampipe | API-backed SQL, parallel connections, cached results, filters and selected columns that can avoid additional API calls | Compare cold and warm queries separately; include field hydration and data age |
| CloudQuery | Source results stream to destinations, which can batch writes; most tables fetch full snapshots, supported incremental tables retain cursors, and the AWS plugin also offers event-triggered selective sync | Compare full sync, cursor sync, and supported event-driven modes separately |
| CloudQuery parallel sync | Non-overlapping shards and unique source identities prevent conflicting writes and stale-row cleanup | Partition ownership and deletion scope must be explicit |
| AWS Config | Aggregates recorded configuration across accounts and regions where Config is enabled | Reuse available recorded data, with explicit recorder/coverage prerequisites |

Sources checked 2026-11-13: [Steampipe FAQ](https://steampipe.io/docs/faq/overview),
[CloudQuery syncs](https://www.cloudquery.io/docs/cli/core-concepts/syncs),
[CloudQuery AWS event-based sync](https://www.cloudquery.io/hub/plugins/source/cloudquery/aws/latest/docs#event-based-sync),
[CloudQuery parallel execution](https://www.cloudquery.io/docs/cli/managing-cloudquery/running-in-parallel),
[AWS Config aggregation](https://docs.aws.amazon.com/config/latest/developerguide/aggregate-data.html).
These describe public behavior, not independently verified competitor throughput.

## Definitions that prevent misleading speed claims

Collection completion means every declared account/region/collector scope has an
explicit result. Denied, unsupported, interrupted, and failed scopes remain visible;
they are not counted as complete coverage. SDK availability is not collector coverage.

Freshness has three clocks: provider change to query visibility; event receipt to
query visibility; and age of the last authoritative successful observation. Report
all available clocks, clock uncertainty, and unobservable provider timestamps.
Never substitute receipt latency for provider-to-CMDB latency.

A resource absent from a failed or partial enumeration is not deleted. Successful
scope completion establishes absence only for that scope and declared resource
population. Resolve source conflicts using identity, ordering, freshness, and
authority rules; preserve the evidence behind the selected representation.

Durable acceptance means an observation and its projection work survive the stated
failure model. Compare identical durability policies; do not gain speed by silently
acknowledging before commit, disabling fsync, or discarding relationships/history.

## Candidate budgets to validate

| Dimension | Proposed measurement / initial target |
| --- | --- |
| Bootstrap | Time to first useful results and full declared scope completion; compare at equal coverage and quota allocations |
| Continuous collection | Sustained valid updates/s and backlog age under declared change rates, including skew and bursts; no unbounded queues |
| Change visibility | Candidate p95 receipt-to-query-visible latency under 5 seconds at the declared steady workload; separately measure provider delivery delay and reconciliation lag |
| Queries | Candidate p95 under 250 ms for a frozen indexed lookup/filter/one-hop suite at 10 million synthetic assets; record rows examined, returned, and freshness |
| UI | Candidate p95 frame time under 16.7 ms during scripted pan/zoom on a bounded 5,000-node/20,000-edge view; initial useful view under 2 seconds on a declared client/network |
| Recovery | Crash/restart, duplicate delivery, expired lease, and projection rebuild preserve accepted observations and converge without manual repair |
| Efficiency | API attempts and estimated charge per valid changed asset, CPU time, peak RSS, write amplification, network bytes, projection lag, and total infrastructure cost |

These numbers are proposed engineering targets, not measured results or an approved
SLA. Budgets apply only to a published workload, hardware, concurrency, permissions,
durability policy, and query suite. Report p50/p95/p99 only with sufficient samples;
three short runs establish a preliminary range and median, not tail latency.

## Architecture hypotheses to prove

1. Discover accounts through Organizations and maintain an explicit onboarding and
   coverage registry. Use dedicated read-only roles, credential refresh, partition
   separation, and per-account permission results. Do not embed access keys.
2. Schedule bounded account/region/service/collector work with durable ownership,
   lease fencing, fairness, and quota coordination across workers. Reuse SDK
   clients and credentials where valid. Limit retries and prevent retry storms.
3. Bootstrap with bulk enumeration; prioritize stale scopes and changed resources
   afterward. Consume useful Config/EventBridge/CloudTrail signals, then reconcile
   periodically. Events alone cannot establish completeness: AWS documents both
   best-effort and at-least-once service delivery behavior.
4. Resume enumerations safely with collector-specific token/epoch rules. Persist
   idempotent observations and projection work atomically. Use bounded batches
   and explicit overload behavior; acknowledge only the promised durability.
5. Keep the current SQLite mode for local use. Evaluate PostgreSQL as the shared
   control/observation store before adding another database. Retain immutable
   evidence/history with measured retention costs. Treat FalkorDB as a rebuildable
   relationship projection and fence graph updates against old observations.
6. Serve indexed persisted read models to the UI. Use server-side filtering,
   authorization, aggregation, pagination, and bounded graph neighborhoods. Stream
   versioned deltas; do not make browser interactions trigger fleet-wide scans.
7. Evaluate WebGL2 rendering with level of detail, typed buffers, and layout work
   off the main thread. Use WebGPU only where supported and measured beneficial.
   GPU rendering does not remove API, transfer, layout, or authorization costs.

Event delivery reference:
[AWS EventBridge service delivery levels](https://docs.aws.amazon.com/eventbridge/latest/ref/event-delivery-level.html).
These are proposed choices; introduce additional storage, SIMD, allocators, or
specialized scheduling only after profiles establish the bottleneck.

## Current implementation boundary

At source commit `57ee7de687be3f445a39bf2e2eb81807e5d68959`, this is a local
inventory CLI with SQLite, a process-local API scheduler, persistent whole-scan
jobs, and a replayable FalkorDB projection. It is not a distributed CMDB service.
The profile input is capped at 128 profiles per invocation; Organizations account
discovery is a separate path, not a proof of fleet capacity.

Current storage source preference can choose an older direct observation over a
newer Config observation. Automatic tombstones and retention are absent. Per-
collector durable enumeration resume, distributed quota coordination, a CMDB web
API, SSO/RBAC, and the GPU UI still require implementation. See `STORAGE.md` for
the existing source/absence semantics. Whole-project strict engineering review
also remains rejected; prior functional checks do not erase those findings.

The release baseline in `CMDB_BASELINE_2026-11-13.md` measures existing local
ingestion and mock scheduling only. No production optimization was made in this
measurement pass.

## Reproducible comparison protocol

Freeze a manifest before each campaign: source/plugin versions and hashes,
resource/field/tag/relationship matrix, account/region population, permissions,
query text, freshness limits, destination indexes and durability, hardware,
network path, worker counts, and AWS quota allocations. Keep unmodified released
competitors and record configuration changes. Separate a common comparable
schema from each product's additional coverage; publish both.

Use three separate comparisons: cold full inventory, steady-state reconciliation,
and interactive queries at equal data age. A cached query cannot be compared with
a fresh full inventory as if they perform the same work. Run order should be
randomized or interleaved, with warm-up, sufficient independent repetitions,
confidence intervals, and raw results. Do not run competing tools concurrently
against shared AWS quotas for a speed comparison.

First local workloads: 1,000 / 3,000 / 10,000 simulated accounts, 1 million /
10 million assets, skewed account sizes, empty regions, global services, varied
payloads, high-degree relationships, duplicate and reordered events, and deletion
bursts. These are planned test sizes, not supported capacity claims. Inject
throttling, expired credentials/tokens, access denial, crashes, slow disks,
disconnects, and stalled graph projection. Publish deterministic fixture seeds
and an independent expected-state oracle.

For AWS comparisons, use explicitly authorized sandbox accounts and matched
scope/permissions. Check final state, fields, identities, relationships, deletions,
source timestamps, and coverage errors before accepting a faster result. Progress
through small real-account trials and a multi-day soak before a thousand-account
rollout. No real AWS collection or role deployment is authorized by local-emulator
setup alone.

Allow performance claims only for named workloads and tested competitors. An
absolute market-wide claim requires broader evidence than one stage benchmark.

## Delivery sequence

1. Accept identity, ordering, freshness, absence, and failure contracts; build the
   deterministic workload/oracle and capture profiles on representative data.
2. Implement one complete vertical slice: account discovery, durable bounded
   collector tasks, shared quotas, idempotent persistence, reconciliation, and
   query-visible updates with explicit coverage status.
3. Prove restart/fencing/duplicate/deletion behavior and throughput with the local
   emulator; replace simulated transport with authorized AWS validation afterward.
4. Add the authorized query API and a bounded GPU graph/table view. Measure actual
   browser frame times, layout/transfer time, query latency, and memory.
5. Expand collector parity, run fair head-to-head campaigns and continuous soak,
   then widen production rollout based on observed limits.

Performance work follows the installed Holzmann latency/throughput and runtime
architecture references: release-mode baselines, profiles before optimization,
bounded resources, explicit durability, correctness gates, and retained evidence.

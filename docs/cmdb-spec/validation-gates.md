# Acceptance behavior and release gates

These are executable-behavior requirements for subsequent test/proof planning.
No scenario below is claimed implemented or passed. The independent test planner
owns detailed commands, fixtures, assertions, fault schedules and evidence paths.
Tests must invoke production domain/service/SDK boundaries, not copied toy logic.

| ID | Given / When / Then acceptance scenario |
| --- | --- |
| A01 | Given two Organizations, standalone accounts, a wrong-account role, a noncommercial identity, and paginated discovery; when onboarding/bootstrap/discovery runs; then only verified commercial targets become enabled, every membership page is accounted for, mutations require separate authority, and missing access is visible. |
| A02 | Given pinned SDK/provider/competitor catalogs with an added type, missing tags and a parent-only table; when the universe is reconciled; then every row has evidence/classification, drift is reported, unsupported differs from unimplemented, and the replacement gate fails on an accepted missing capability. |
| A03 | Given a previously populated account with denied describes, partial region discovery and later account removal; when collection repeats; then prior inventory persists with explicit stale/partial/visibility-lost state, no false deletions occur, and healthy accounts continue. |
| A04 | Given shared resources, non-ARN IDs, reused IDs, global services and ambiguous network owners; when observations join; then canonical identity/incarnation rules are deterministic, global work does not duplicate by region, and unresolved ownership never becomes a verified fabricated ARN. |
| A05 | Given empty nonterminal pages, token cycles, invalid tokens and parent failures; when collection crashes/restarts at each page boundary; then complete valid pages/checkpoints/outbox are atomic, exact replay is idempotent, conflicting replay fails and incomplete populations cannot complete. |
| A06 | Given two real PostgreSQL workers and fault points before/after claim, commit, ACK, renewal, expiry and cancellation; when ownership changes or connections fail; then one current generation commits, ambiguous commits resolve by key, stale owners fail, and recovery preserves accepted state without claiming exactly-once AWS reads. |
| A07 | Given old direct/new Config data, late old capture times, duplicate/reordered events, redacted fields and source conflicts; when projection replays in different arrival orders; then declared comparable-order rules converge, uncertainty remains explicit, no priority-only stale overwrite occurs, and provenance/timestamps remain correct. |
| A08 | Given two separated complete epochs, intervening denial, a newer observation after cutoff, deleted/recreated identity and partial edge reads; when absence is evaluated; then only qualifying evidence confirms deletion, denial does not advance evidence, newer resources survive, and history/other-source edges remain intact. |
| A09 | Given projection gaps, duplicate/obsolete batches, graph outage and rebuild; when outbox delivery resumes; then versions cannot regress, watermark does not skip gaps, inventory survives, lag is visible, and validated namespace publication/replay matches the authoritative oracle. |
| A10 | Given shared account/service quota domains across workspaces, throttles, retry hints, credential refresh and owner failover; when independent workers run; then every attempt is budgeted, cooldown survives transfer, overlap stays within the documented bound and no retry layer multiplies the budget. |
| A11 | Given authenticated/forged, duplicate, unknown, oversized, delayed and poisoned event inputs; when accepted receipt/refresh processing runs; then durable-before-ACK and safe quarantine hold, coalescing preserves linkage, successful T4-T1 meets the declared load objective and every failure/miss remains visible. |
| A12 | Given the fleet profile with hot-account events and backfill; when collection/UI/projectors run together for multiple hourly cycles; then every healthy expected scope completes within 3,600 seconds and no workload class hides or starves overdue scopes. |
| A13 | Given a full queue, disk pressure, slow database and offered load above the ceiling; when intake continues; then admitted data remains durable, memory/work stay bounded, unsafe acknowledgements/drops do not occur, and explicit overload/freshness breaches recover under the runbook. |
| A14 | Given authorized API users, stable snapshots, expiring cursors and concurrent updates; when searching/paginating/traversing/reporting/cancelling; then results are bounded, correctly versioned and authorized, streams resume/reset safely, and 202 is distinguishable from completion. |
| A15 | Given two editors reading revision 5 and a tag-derived owner; when both edit and an override is later removed; then one update receives revision 6, the other conflicts without loss, audit retains attribution, and provider observations/AWS state remain unchanged. |
| A16 | Given malicious SQL/CTEs/functions, hidden accounts, cross-workspace edges/counts, pooled connections and revoked tokens; when every read/export/stream/cache path is exercised; then no unauthorized row/field/count leaks, roles cannot bypass storage policy, and revocation invalidates access. |
| A17 | Given canary secret strings, tag markup, forged origins, decompression bombs, webhook SSRF and export traversal; when ingestion/API/UI/export/backup paths run; then secret/unsafe content does not persist or execute, bounds hold, TLS/session controls apply, and audit/errors remain safe. |
| A18 | Given export retries, partial publication, receiver timeouts/429s and revoked download access; when delivery replays; then immutable counts/hashes/manifests verify, receiver idempotency is preserved, retries terminate visibly, and credentials/unauthorized data do not leak. |
| A19 | Given old SQLite exports, supported/newer schemas, old/new workers and a failed migration; when importing/upgrading/canarying/rolling back; then declared compatibility/provenance/counts hold, unsafe combinations fail explicitly, and migration rollback boundaries are documented and exercised. |
| A20 | Given expired observations, held records, unverified archives and a replay older than receipt dedup retention; when retention/archive/replay runs; then no unverified/held evidence is deleted, tombstone safety prevents resurrection, and retained history restores with verified manifests. |
| A21 | Given populated/empty/denied/stale/unresolved views, network loss, expired stream, late layout result and GPU context loss; when a real user navigates/table-filters/expands/edits by mouse and keyboard; then states are distinct, the UI recovers without unauthorized caches, accessibility works and resources remain bounded. |
| A22 | Given frozen functional/fleet/stress fixtures and declared hardware/quotas; when ingest/reconcile/event/query/projection/retention run together; then counts match the oracle, queue/memory bounds hold and the measured supported envelope/cost is reported without extrapolated capacity. |
| A23 | Given the named reference client/read model and scripted queries/pan/zoom/selection with labels; when enough samples run; then query/frame/input/first-view budgets are measured separately including transfer/layout/GPU and reported with errors and tails. |
| A24 | Given the exact release candidate; when strict source/contract/behavior/proof-lane/manual/adversarial/independent reviews execute; then evidence maps to production and source hashes, unexecuted lanes remain blocked, rejected findings remain rejected, and only passing required evidence permits release. |
| A25 | Given a clean host/cluster and isolated local emulator; when installing/onboarding/upgrading/decommissioning using published instructions; then pinned dependencies start correctly, real credentials remain protected, realistic SDK wire behavior is exercised, and documented daily workflows actually work. |
| A26 | Given injected quota collapse, lost credentials, projection gap, storage pressure and missed deadlines; when alerts fire; then dashboards identify affected scope and age, correlation reaches the failing stage, cardinality remains bounded and the stated runbook resolves the condition. |
| A27 | Given abrupt process/node/zone failure, partitioned primary, disk-full and lost site; when failover/restore/catch-up is executed; then accepted-state guarantees and fencing hold for the declared failure model, recovery validates state and measured RPO/RTO match the published profile. |
| A28 | Given small authorized real-account pilots followed by progressively larger deployments; when a seven-day continuous soak runs with expected change/deletion/error activity; then coverage, freshness, resource growth, recovery and operator workflows satisfy the declared gates before thousand-account rollout. |
| A29 | Given pinned released competitors and equal populations/fields/age/permissions/quotas/durability; when cold/warm/full/incremental/event-supported campaigns run in fair order; then state equality precedes speed comparison, raw evidence and uncertainty remain available and claims name their tested scope. |

## Engineering gates

Require formatting, warnings-denied Rust checks/builds/tests, locked dependency
and SDK provenance validation, schema generation checks, real PostgreSQL migration/
concurrency tests, API contract compatibility, UI interaction QA, dependency/SBOM
review, source bounds and immutable artifact identity. Retain exact commands,
exit statuses, tool versions, hashes, seeds, logs and limitations. A zero-test
command is not behavioral evidence; compile-only SDK coverage is not runtime proof.

The installed strict policy requires Rust files at most 300 lines, functions at
most 25 physical lines, at most five parameters, decomposed async protocols,
private-field parsed domain types, semantic modes rather than behavior booleans,
no first-party unsafe/unwrap/expect/panic/todo/unimplemented paths, and bounded
capability-gated effects. Test assertions must prove public behavior. Do not game
size rules with numbered fragments, macros, deleted assertions or hidden waivers.

Apply the Rust-contract -> proof planning/review -> implementation-bound evidence
pipeline. The default Rust-local lane profile considers Verus, Kani, Flux RS and
proptest, with Loom for interleavings and fuzzing for hostile/parser boundaries.
The specialist planner/reviewer must decide applicability and concrete obligations;
missing required tooling blocks its claim. Copied models require explicit bridges;
assumptions/stubs/trusted code/bounds remain visible. Proof seeds below are not
proof obligations and no formal verification has been executed for this spec.

Manual QA invokes real CLI/API/UI workflows, not only tests. Red Queen campaigns
use deterministic saved seeds, evolved failure sequences and held-out scenarios;
report actual generations/cases and regressions rather than an invented score.
Independent Black Hat/async/DDD/architecture and evidence reviews cannot be
self-approved by the implementation writer. The existing strict rejected report
is historical input; its outstanding findings must be resolved/reviewed against
the final source, never overwritten by this design document.

When Rust work is delegated locally, use the user-authorized OMP CLI bridge to
the actual RTX 3090 worker, require its mandatory skill reads, isolate changes,
retain route/transcript/source evidence, and independently test the candidate.
No GPU worker ran or approved this specification. Do not label earlier rejected
worker candidates accepted. No worker starts midflight builds contrary to the
local-GPU skill; the parent owns integration and verification.

## Release progression, not atomic task decomposition

Specification/contract acceptance -> one end-to-end distributed vertical slice ->
local deterministic/fault qualification -> authorized 2/20/100-account pilots ->
collector/universe completion plus UI/security/operations qualification -> mixed
load and fair comparisons -> seven-day soak -> controlled thousand-account
rollout -> replacement release. Every stage retains its own acceptance pin.
Detailed implementation/proof/test task decomposition is a subsequent workflow.

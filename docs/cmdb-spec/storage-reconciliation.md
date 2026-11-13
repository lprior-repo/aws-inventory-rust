# Storage, reconciliation, history, and projections

## Required logical records

| Record | Key / mandatory controls |
| --- | --- |
| Workspace/source/account | Workspace-scoped IDs, verified account, role binding reference, membership/policy revision |
| Collector definition/capability | Version, population digest, field/permission/event/deletion policies and evidence |
| Work/lease/attempt | Work ID, unique logical intent key, generation, owner/expiry, bounded outcome and next eligibility |
| Scope epoch/page | Scope digest + epoch + page sequence; canonical fingerprint, tokens, cutoff, completion evidence |
| Event receipt/refresh intent | Source binding + provider receipt identity, payload digest, accepted sequence, coalescing linkage |
| Resource identity/alias/incarnation | Workspace + canonical identity; separately versioned alias/continuity evidence |
| Observation/field state | Immutable observation ID, source, capture/receipt/observation times, visibility, redaction and schema |
| Edge observation | Typed endpoints, relation identity, source population, evidence, confidence, lifecycle version |
| Current state | Resource incarnation + selected version, freshness/conflict/deletion state and field provenance |
| Coverage/absence evidence | Expected scope, terminal result, age/deadline, qualifying epochs and exclusion reasons |
| Outbox/projection cursor | Accepted sequence + projection identity; idempotency key, watermark and retry disposition |
| Annotation/service/owner | Revisioned enterprise values and provenance, separate from provider observations |
| Export/delivery/audit | Snapshot watermark, content hashes, actor/policy revision, idempotency key and safe delivery status |

Indexes support workspace+identity, account+region+kind, indexed tags/owner/service,
scope+epoch, pending work eligibility, active leases, outbox cursor, and historical
identity+time ranges. Partition high-volume observations/receipts/audit by measured
time/workspace needs. Avoid blanket JSON indexing; publish measured write/query
tradeoffs. Query replicas expose replay lag and cannot claim a primary watermark
they have not applied.

## Atomicity and acknowledgement

Page acceptance commits resource/edge observations, fingerprint, checkpoint,
coverage effects, and outbox in one PostgreSQL transaction. Receipt acceptance
commits the receipt and refresh intent in one transaction. Conditional updates
check work generation and epoch status in that transaction. Unique constraints
enforce idempotency; application checks alone are insufficient.

Keep transactions short and free of AWS/network effects. Resolve ambiguous
commit outcomes by idempotency lookup. Read queries that require a stable page
set use a retained snapshot/read-model version or documented consistent cursor;
multiple read-committed statements are not automatically one stable snapshot.

## Source and field reconciliation

Retain every accepted source observation. Select fields through a versioned
policy using authoritative identity, source capture validity, scope visibility,
freshness, comparable ordering, and field-specific authority. Source preference
breaks a valid comparable tie; it does not permanently outrank newer credible
evidence. Missing, denied, absent, redacted, unknown, and explicit null differ.

A delayed Config record with an old capture time cannot become newly fresh just
because it arrived now. A direct response without comparable provider ordering
cannot erase a credible later deletion event blindly; schedule confirmation and
expose uncertainty. Database sequence orders local projection application, not
provider truth. Each hybrid field retains its own provenance and effective time.

Current selection must be deterministic on replay for the same observations and
policy version. Reprocessing after a policy change creates a new projection
version and records the policy change; it does not rewrite original evidence.

## Confirmed absence

Absence requires exact complete scope evidence, matching visibility policy,
and a cutoff older than the resource's last relevant observation. Two successful
enumeration epochs separated by the reconciliation interval confirm disappearance;
reviewed resource-specific policies may use equivalent authoritative evidence.
Denied or partially enumerated parents cannot delete children. An event hint is
not confirmation by itself. Account removal is visibility loss.

Edge disappearance requires a complete relationship set from its own source.
Deleting a resource preserves historical edges and identities; current views
mark lifecycle and unresolved references explicitly. Another collector's edge
cannot be erased by an unrelated field/detail read.

## Projection consistency

The shared observation store is authoritative. Current SQL state, graph views,
search indexes, archives, and external sinks are derived. Every change carries
entity version and accepted sequence. Idempotent replay applies only newer
entity versions; global watermarks advance only through a contiguous applied
prefix or explicitly tracked gaps. A maximum seen sequence is not a completeness
watermark. Rebuild in a separate namespace, validate counts/state hashes, then
atomically publish the ready version. Keep the old projection for rollback.

FalkorDB outage retains outbox work subject to explicit storage bounds. SQL/API
responses expose graph lag and can use supported SQL relationship queries;
they cannot silently describe a stale graph as current. Graph deletion/version
operations must be idempotent under ambiguous delivery and obsolete retries.

## Retention and migration

Queryable and archive durations are workspace policy values fixed in the decision
register. Retain stable identity/tombstone summaries and required deletion audit
beyond bulk observation expiry where necessary to prevent stale replay resurrection.
Deduplication and replay horizons must cover source retention/redelivery windows.
Expired old receipts cannot resurrect state merely because their dedup row expired.

Archive manifests bind schema, time/sequence range, row counts, hashes, encryption
key reference, and successful verification. Delete hot evidence only after the
archive is verified and recovery/retention policy permits it. Holds override
ordinary expiry. Retention is not a backup; backups are not a query archive.

Migrations use expand/migrate/contract with compatibility windows and bounded
backfill. Export/import current SQLite observations with schema/provenance and
source fingerprints into PostgreSQL, then verify counts and identity/state hashes.
Never run an irreversible migration automatically from a read-only user query.
Fail explicitly for newer/foreign schemas; do not fabricate compatibility.

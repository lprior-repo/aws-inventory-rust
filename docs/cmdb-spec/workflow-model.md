# Workflow model

Each arrow is a guarded transition. Network reads are at least once; accepted
state changes are idempotent and fenced. No exactly-once AWS execution is promised.

## Source onboarding

Discovered -> AwaitingAccess -> Verifying -> Enabled.
Verification checks caller identity, commercial partition, expected account,
allowed role, endpoint selection, region discovery, and capability evidence.
Failure -> Degraded/AwaitingAccess with a typed reason. Operator suspension ->
Suspended. Complete Organizations evidence of removal -> Removed registration
membership, not deletion of every formerly visible AWS resource.

Bootstrap rendering of role/event infrastructure produces a reviewable plan.
Applying it is a distinct administrator action and audit event. Inventory
credentials cannot create IAM roles, trails, recorders, indexes, or event rules.

## Enumeration

Planned -> Running(epoch, cutoff, population_digest).
For each page: admit bounded work -> obtain quota permission -> read provider ->
parse/normalize -> accept atomic page/checkpoint/outbox -> continue from token.
An empty nonterminal page continues. An empty terminal page can complete an
otherwise valid enumeration. A page replay must match its canonical fingerprint.

Running -> Complete only after every required parent population and page chain
has terminal evidence. All other terminal conditions yield Incomplete,
Cancelled, or Superseded with committed progress preserved.

Expired/invalid provider tokens follow the collector's reviewed restart rule:
start a new epoch with a new cutoff, retain old observations/checkpoints, and
deduplicate idempotently. Never append a restarted first page to an old chain.

## Event-driven refresh

Untrusted input -> authenticated source binding -> bounded validated envelope ->
durable receipt/refresh intent transaction -> source acknowledgement.
Duplicate receipts retain deduplication accounting. Multiple hints for one
resource coalesce into bounded pending refresh work without losing evidence of
which receipts were accepted. Event-to-resource mappings are versioned.

The worker performs a reviewed targeted read or constrained re-enumeration.
It commits observations/checkpoint/outbox, then advances processing progress.
Unknown mappings enter quarantine and schedule bounded reconciliation; they do
not fabricate complete state or spin indefinitely. Event source identity never
overrides the target account/role identity check.

## Work ownership

Ready -> Leased(owner, generation, expiry) through one conditional transaction.
Renewal requires the same owner and generation. Completion and checkpoint
mutation require current ownership and expected generation. An expired owner
cannot finalize work; a replacement receives a strictly newer generation.

Lease loss -> stop dispatch -> cancel effects -> bounded drain -> explicit
terminal/retry disposition. Reads already sent to AWS may still finish. Durable
page acceptance checks the lease/epoch fence, so obsolete ownership cannot
publish stale current state. Recovery may repeat provider reads.

## Absence and deletion

Observed -> SuspectedAbsent only from a complete successful enumeration of the
exact authoritative population and visibility class. The resource must predate
the epoch cutoff; concurrent newer observations prevent false absence.

SuspectedAbsent -> ConfirmedDeleted requires two independent complete successful
scope epochs separated by the configured reconciliation interval, or a reviewed
resource-specific confirmation policy with explicit equivalent evidence.
Failed/denied/incomplete epochs do not advance the confirmation counter.
Any authoritative reobservation resets contradictory absence evidence.

ConfirmedDeleted -> Reobserved retains the tombstone and resolves whether the
resource is a new incarnation. Conflicting or ambiguous provider identity remains
explicit. Retention removes evidence only under its separate audited policy.

## Relationship lifecycle

A complete relationship population can supersede its own prior edges. A partial
describe or missing target cannot erase other sources' edges. Deleted endpoints
remain historical identities; current views omit or mark tombstoned assertions
according to query mode. Unresolved targets remain distinct from deleted ones.

## Projection and API delivery

Observation transaction -> durable outbox -> SQL/current-state projection ->
graph/search/export projection -> versioned query visibility acknowledgement.
Projectors are independently replayable and idempotent. Graph outage never
prevents observation acceptance while bounded outbox capacity remains available.
When capacity is exhausted, backpressure is explicit; success is not invented.

API subscriptions carry a sequence cursor and workspace authorization version.
Resume from retained cursor or return CursorExpired with a required snapshot
refresh. Revocation closes access and invalidates dependent streams/caches.

## Shutdown

Stop intake -> cancel producers -> drain accepted bounded work -> commit or
explicitly abandon uncommitted transactions -> finalize lease/checkpoint outcome
-> close stores and transports. A hard deadline reports incomplete drain and
preserves durable work for recovery; it never reports a clean complete run.

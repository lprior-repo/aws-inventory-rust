# Core preconditions, postconditions, and invariants

## Preconditions

P-01: A source registration names a commercial account/Organization and reviewed
role; verified caller identity matches the target before accepting observations.
P-02: An effect has a current capability, remaining byte/time/attempt budget,
and a valid work lease/epoch. Provider credential acquisition is separately bounded.
P-03: Every accepted page matches an immutable scope population and continuation
chain, or is an exact previously committed replay.
P-04: Every query/edit has a verified principal, current workspace authorization,
and a declared bounded result/execution policy.

## Postconditions

C-01: Successful page acceptance atomically persists observations, provenance,
page identity/fingerprint, checkpoint, coverage updates, and projection intent.
C-02: Successful receipt acceptance persists the causal receipt and refresh
intent before acknowledging the upstream transport.
C-03: Successful epoch completion produces CompleteScopeEvidence for the exact
population; a partial or denied page cannot satisfy this postcondition.
C-04: Successful projection application advances monotonically or is an exact
no-op replay. Query results expose the actual projection watermark.
C-05: Confirmed deletion preserves history and carries the accepted confirmation
policy/evidence. A later observation cannot silently erase that history.
C-06: Successful annotation edits condition on the expected revision and append
an audit event in the same transaction.

## Invariants

I-01: Workspace authorization applies to rows, fields, graph edges, aggregates,
exports, subscriptions, caches, logs, and indirect counts.
I-02: AWS inventory credentials perform configuration reads only. Bootstrap
mutation uses separate explicit administrator authorization and credentials.
I-03: Source/owner/observation time/provider time/receipt time remain distinguishable.
I-04: Denial, timeout, cap exhaustion, missing permission, and unsupported APIs
are not evidence of emptiness, absence, or complete coverage.
I-05: A stale lease cannot commit new accepted work/current-state transitions.
I-06: Event and page replay cannot duplicate logical state or regress a projection.
I-07: Current-state selection does not permanently prefer stale direct data over
fresh credible evidence merely because of a fixed source priority.
I-08: Provider chronology is not inferred solely from local arrival order. Unknown
or incomparable source ordering triggers explicit uncertainty/refresh policy.
I-09: No absence sweep affects resources first observed or updated after its
epoch cutoff, another source population, or another principal visibility class.
I-10: Deletion requires the confirmed policy; partial scans never advance it.
I-11: Relationship completeness is independent of resource enumeration completeness.
I-12: Pending work, in-memory data, graph traversals, HTTP bodies, retries, and
concurrent effects have finite configured bounds and observable saturation.
I-13: Every provider attempt is admitted under the applicable quota policy;
workspace separation cannot evade a shared AWS account/service quota.
I-14: A query never labels stale/partial data complete or fresh without evidence.
I-15: Durable accepted observations survive the promised storage failure model;
projection outage cannot falsely acknowledge query visibility.
I-16: Operator changes, identity changes, edits, exports, retention, and access
policy changes are audited with attribution and policy/version context.
I-17: Schema/collector changes preserve declared compatibility or require an
explicit migration/version boundary. Old observations retain their schema version.
I-18: Cancel/drain/finalize reports what actually completed; a drain timeout is
not a successful completed scan.
I-19: Sensitive fields are classified/redacted before durable generic storage,
logging, graph projection, caches, or export; resource tags are untrusted input.
I-20: Design documents, successful compilation, emulator results, and SDK
availability are not production scale, independent review, or proof acceptance.

## Evidence boundary

The current local CLI implements portions of these concerns but does not satisfy
this complete CMDB contract. The implementation must map each invariant to
production source, independent behavioral evidence, and applicable verification
lanes. Domain proof seeds are hints, not approved plans or executed proofs.

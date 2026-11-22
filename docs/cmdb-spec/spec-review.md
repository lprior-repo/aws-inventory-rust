# Specification consistency review

Review type: author consistency review, not an independent Black Hat approval.
Status: DRAFT / UNVERIFIED implementation. Final policy/pre-mortem decisions are
open in `decisions.md`. There is no claim of production readiness or proof.

## Checks and corrections

- The user's commercial-only launch decision overrides the earlier proposal to
  design a broader partition launch. Unsupported partitions are explicit.
- Hourly reconciliation and five-second processing replace the earlier relaxed
  15-minute-critical/daily-full proposal. Failure stays in the reported denominator.
- Healthy scope progress is independent; no whole-fleet atomic publication barrier.
- Two complete separated epochs or reviewed equivalent confirmation govern
  deletion; no partial/denied/account-loss inference is permitted.
- Failed/unknown event processing cannot satisfy the five-second freshness SLO
  merely by returning a fast failure. Clock starts before queue admission.
- Graph/SQL required projection lag is included in query visibility; browser
  transfer/frame time and provider delivery remain separately measured.
- CloudQuery event-triggered selective syncing is included in competitor
  comparisons; generic full-table behavior is not the only comparison mode.
- SDK operation inventory was refreshed: old audit 19,617 versus current 19,643
  modules, with 22 changed SDK versions. New seed rows use the current pin and
  exact archive checksums; no old snapshot is called current evidence.
- Shared quotas follow actual AWS account/service domains, including when an
  account is observed in more than one workspace. Lease fencing does not claim
  to revoke already-sent provider requests or create exactly-once AWS effects.
- State ordering, receipt time, capture time and provider chronology are separate;
  fixed source priority and max-seen outbox sequence are insufficient.
- SQL authorization is enforced through real privileges/context and parsed scope;
  read-only text prefixes or owner-bypassing RLS are not accepted isolation.
- Retention is separate from backup and must preserve replay/tombstone safety.
- The GPU UI is bounded and accessible; it never assumes full-estate browser load
  or a universally available WebGPU device.
- Existing rejected strict review findings remain explicit; this design does
  not turn historical compilation/tests or rejected GPU work into acceptance.

## Remaining design review focus

Finalize overload/retention policy, complete the operator pre-mortem and recovery
objectives, then independently challenge capacity feasibility and shared quota
ownership under pauses/failover. The coverage universe still requires pinned
competitor snapshots and semantic classification; the raw SDK inventory is an
honest starting surface. Benchmark integration decides the concrete UI/renderer.
These are explicit design/qualification gates, not hidden implemented features.

Subsequent planning must assign production source, independent behavior tests,
proof-lane dispositions and evidence paths to all 135 traceability rows. They
currently have empty source/test/evidence references by design, rather than
inventing nonexistent implementation or claiming a completed proof plan.

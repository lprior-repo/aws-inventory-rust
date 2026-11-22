# Continuous AWS CMDB — full specification review draft

Version 0.1, 2026-11-13. Status: comprehensive design draft; not implemented,
production-qualified, independently approved, or the final accepted baseline.
Pending policy/pre-mortem decisions are explicit in [decisions](decisions.md).

Build a self-hosted enterprise AWS CMDB for thousands of continuously monitored
commercial accounts. Target received-change visibility within five seconds and
successful hourly reconciliation of every supported expected scope. Preserve
last known state with visible coverage/freshness, require confirmed deletion with
history, and provide SQL/report/export plus a performant GPU-assisted browser UI.
Replace the accepted AWS inventory workflows of Steampipe/CloudQuery through a
reviewed coverage universe and reproducible equal-work performance comparisons.

## Read in this order

1. [Confirmed decisions, policy choices and finalization boundary](decisions.md).
2. [Verified starting point and compatibility boundary](current-state.md).
3. [135 normative EARS requirements](requirements.md).
4. [Coverage universe and collector seed inventory](coverage.md).
5. [Domain language and aggregate ownership](domain-model.md).
6. [Preconditions, postconditions and invariants](contract.md).
7. [Trusted type contracts](type-contracts.md).
8. [State machines and legal transitions](workflow-model.md).
9. [Owned error taxonomy and retry dispositions](error-taxonomy.md).
10. [Pure/async/storage/network/security boundaries](boundary-map.md).
11. [Distributed collection, event intake, quotas and deadline scheduling](distributed-collection.md).
12. [Storage, reconciliation, history and rebuildable projections](storage-reconciliation.md).
13. [API, SQL, integrations, identity and enterprise security](api-security.md).
14. [Browser workflows, GPU rendering and accessibility](ui.md).
15. [Deployment, telemetry, runbooks, recovery and upgrades](deployment-operations.md).
16. [Fleet workload, freshness and performance acceptance](performance-acceptance.md).
17. [29 acceptance scenario families and release gates](validation-gates.md).
18. [Failure hazards and cascading consequences](hazard-analysis.md).
19. [Primary references and skill/evidence boundaries](sources.md).
20. [Specification consistency review](spec-review.md).

The machine-readable requirement, traceability, proof-seed and seed-register files
were removed from the repository. The prose in this document set is not executed
proof or implementation acceptance.

## Implementation handoff boundary

Finalize pending policy choices and architectural pre-mortem before treating the
baseline as accepted. Then use the architecture-to-beads workflow to decompose
this contract into independently reviewable implementation work, with proof/test
planning and evidence gates. This pass does not create atomic tasks or beads.

The replacement release is not complete until its coverage universe, functional,
security, recovery, UI, continuous-load, real-AWS and independent review gates
pass on the exact source and deployment artifacts. No single benchmark or clean
compile establishes completion.

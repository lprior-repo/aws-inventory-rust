# Decisions, assumptions, and finalization boundary

## Confirmed user requirements

| ID | Decision | Evidence |
| --- | --- | --- |
| D01 | Self-hosted enterprise platform, multiple Organizations/accounts and isolated workspaces; no public multi-tenant SaaS launch requirement | User scope answer, 2026-11-13 |
| D02 | Commercial AWS only | User partition answer, 2026-11-13 |
| D03 | Received changes processed within five seconds; every supported scope reconciled hourly | User aggressive-freshness answer, 2026-11-13 |
| D04 | Independent scope progress, retained last-known inventory, explicit stale/incomplete coverage | User consistency answer, 2026-11-13 |
| D05 | Confirmed deletion with history retained; two separated complete epochs or reviewed resource-specific confirmation | User deletion answer, 2026-11-13 |
| D06 | Thousands of accounts continuously, with a WebGL or similar performant UI and replacement of AWS Steampipe/CloudQuery workflows | Earlier user instructions in this chat |
| D07 | Highest measured performance is a product objective; unsupported "fastest" claims are prohibited | Objective translated into the equal-work performance contract |

## Pending user policy choices

| ID | Proposed default / unresolved choice | Effect |
| --- | --- | --- |
| P01 | Preserve accepted changes, bounded backpressure/coalescing, visible freshness breaches; no deliberate accepted-event dropping | Defines overload acknowledgement/durability behavior |
| P02 | 90 days queryable configuration history plus one year archived, configurable by workspace | Defines initial retention and capacity/cost calculations |

These two questions are pending in the chat. They are not approved because time
elapsed or because a suggested option is preselected. Draft chapters consistently
use the proposed correctness-first overload model. Retention durations remain
policy parameters until answered. If a different answer is selected, revise all
affected requirements/contracts/scenarios together before finalizing the baseline.

## Engineering proposals requiring qualification

- PostgreSQL shared durable work/observations/outbox; SQLite standalone mode;
  FalkorDB rebuildable graph; object storage history/export/backup.
- Kubernetes enterprise deployment and isolated local container composition.
- Rust/Wasm browser application, WebGL2 baseline, optional measured WebGPU;
  final renderer/UI-library selection is a bounded integration benchmark decision.
- 3,000-account / 10-million-resource / 30-million-edge reference profile;
  10,000-account stress characterization and the proposed reference hardware.
- 99.9 percent of accepted supported hints query-visible within five seconds
  under healthy declared load, while every miss remains visible. This percentile
  operationalizes the user's deadline; it is not separately user-approved.
- Initial payload, request, retry, lease, query and graph bounds in the chapters.
  These require collector-specific validation, not blind enforcement over legal
  provider payloads with a false complete result.

## Recovery policy still to harden

Proposed HA objective: no loss of acknowledged accepted state for a single
node/zone loss when durable replication is healthy, and service recovery within
five minutes. Proposed site-disaster objective: recovery point within five minutes
and core service recovery within one hour, followed by measured projection/backlog
catch-up. These are draft targets requiring operator/cost decisions and recovery
drills, not current guarantees. Region-wide zero data loss is not implied.

The final architectural pre-mortem must explicitly challenge those failure
assumptions and identify accountable operators, alert destinations and backup
location/residency. Installation inputs also include IdP/group mapping, DNS/TLS,
storage/keys, resource/cost ceilings, network reachability and authorized AWS roles.
The spec does not require credentials or deployment changes to be supplied now.

## Status vocabulary

Specified is not implemented; implemented is not emulator-verified;
emulator-verified is not AWS-verified; AWS-verified is not fleet-qualified;
fleet-qualified is not independently approved; a benchmark is not a market-wide
ranking. All new requirement traceability begins specified/unverified.

## Local architectural skill boundary

The installed `arch-design-qa` skill requires five phases: EARS, domain contracts,
failure inversion, consequence tracing, and pre-mortem. Its handoff instruction
is: "Only once YOU are satisfied that the architecture is hardened and
battle-tested conceptually" write the final architecture spec.

This bundle is a comprehensive review draft while policy answers and the final
pre-mortem remain open. It is not falsely stamped as an accepted final baseline.
No implementation tasks/beads, production Rust, proof harnesses or independent
review approval were created by this specification-writing pass.

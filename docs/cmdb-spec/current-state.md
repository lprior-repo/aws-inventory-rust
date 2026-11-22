# Verified starting point and migration boundary

Source inspected: `57ee7de687be3f445a39bf2e2eb81807e5d68959`, private repository
`lprior-repo/aws-inventory-rust`, current main checkout. New performance/spec
documents are uncommitted working documents at intake; source has not been
changed by this specification pass.

| Area | Existing evidence | Required extension |
| --- | --- | --- |
| Product | Rust CLI, direct/supplemental collection, exports, SQL analysis | Continuously operated enterprise CMDB/API/UI |
| SDK | 437 exact pinned SDK clients in optional availability catalog | Classify inventory surface; collector implementation/qualification per row |
| Native collectors | CLI catalog reports 32 resource/component kinds | Full accepted universe, enrichment/tags/policies and relationships |
| Cloud Control | 397 roots + 21 parent types, total 418 catalog types | Per-type/region/handler/field qualification and missing universe |
| Persistence | SQLx SQLite schema 6; atomic pages, provenance/checkpoints | PostgreSQL shared store/control/outbox, compatibility import and HA |
| Scheduling | Process-local attempt budgets and bounded streams | Shared quota ownership, deadline fairness and distributed resume |
| Durable jobs | Leased whole-scan local SQLite queue, explicit at-least-once boundary | Per-collector segment/refresh work with cross-host fencing |
| Current selection | Fixed direct > Cloud Control > Config > Explorer source priority | Freshness/ordering-aware field provenance and conflict policy |
| Lifecycle | No automatic tombstone/retention job | Confirmed absence, incarnation history, safe expiry and archive |
| Relationships | Replayable FalkorDB projection and explicit unresolved endpoints | Continuous versioned projection, safe removals and gap-aware watermarks |
| Enterprise access | Local CLI credential context | OIDC, workspace/field authorization, audit, API/session isolation |
| UI | No CMDB browser application | Full enterprise surfaces plus bounded GPU graph/table experience |
| Scale | Local stage benchmark and existing local/emulator functional evidence | Fair head-to-head, real-AWS pilots, concurrent fleet load and soak |

## Evidence provenance checks

The CLI catalog was invoked in this pass and reported AWS support enabled,
32 native kinds, 418 Cloud Control types and schema 6. The current SDK manifest
pins `release-2026-11-12`, official revision
`c8e11bea896b54530c587c29ce30ebd6619af62a`.

The older `artifacts/sdk-capabilities-v06.json` pins release-2026-11-11 and
differs from the current manifest for 22 service versions. It is not the current
operation-register authority. The new offline source-audit result and manifest
hash checks in `catalogs/` must be used for this specification's seed inventory.
SDK source inspection remains static evidence, not runtime service coverage.

The prior strict whole-candidate review (its file was removed from the repository)
is rejected. It identifies long
functions/signatures, primitive domain identities, behavioral booleans, and
unfinished async outcome/observability obligations. It is a dated snapshot;
do not present its old counts as a newly executed source audit, or overwrite its
rejection with this design. Those concerns remain explicit release work.

The preliminary release benchmark measured median roughly 25,035 synthetic
resource rows/second with one edge each and batches of 100, on three 1,000-row
runs with substantial storage variance. It is neither AWS throughput nor query,
UI, fleet, recovery or competitor evidence. The baseline source pin and binary
hash are retained in the earlier performance report.

## Compatibility and transition

Preserve original ZIP/source evidence and the working local installation. Add new
schemas/DTOs through explicit versions and adapters, not silent reinterpretation
of existing scan IDs, timestamps, source priorities or export files. Current
read paths that can upgrade a SQLite file are not the required future read-only
query migration contract; the enterprise migration interface must be explicit.

Keep existing local/emulator workflows operational while the distributed vertical
slice is developed separately. Do not migrate live credentials, deploy new AWS
roles, or scan real accounts as a side effect of writing this spec.

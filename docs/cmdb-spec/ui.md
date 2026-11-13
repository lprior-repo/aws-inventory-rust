# Browser application and GPU visualization

## Required product surfaces

1. Fleet overview: account/OU coverage, resource counts, change backlog, missed
   hourly deadlines, stale data, denied scopes, event lag and projection lag.
2. Account/source onboarding: verified identity, role access, region/capability
   state, infrastructure plan/status, and actionable permission failures.
3. Inventory explorer: server-side filter/search, virtualized sortable table,
   saved views, tag/owner/service facets, explicit null/unavailable distinctions.
4. Resource detail: current configuration, per-field source/capture time,
   provenance evidence, history/diff, lifecycle, unresolved relationships,
   ownership/service annotations, and last successful scope evidence.
5. Dependency graph: bounded neighborhood expansion, group/aggregate views,
   relation/source filters, search/focus, confidence and unresolved markers,
   historical/as-of mode, and export of the authorized visible result.
6. CMDB management: services/applications, owners/teams, custom fields, user edges,
   saved reports and revision conflicts with recoverable editing workflows.
7. SQL/report console: documented schema, bounded read-only execution, parameters,
   coverage/freshness metadata, cancellation, saved queries and asynchronous export.
8. Operations/admin: scheduler, quotas, event intake, failed work/replay, policy,
   identity mappings, audit, retention/holds and supported backup/restore status.

## Rendering contract

Rust/Wasm application logic with a WebGL2-capable renderer is the baseline.
WebGPU is optional capability negotiation, not a launch requirement. Select the
Rust UI/renderer implementation through a measured integration decision before
building the full UI: compare table accessibility, GPU frame time, payload size,
memory, layout latency, maintenance and licensing on the same fixture. A narrow
binding to an established renderer is allowed only with a recorded justification;
first-party domain/layout code remains Rust. No renderer is assumed fastest.

Use typed dense render buffers, stable GPU IDs, batched draws, frustum/viewport
culling, degree aggregation, level of detail and bounded label density. Do not
download or draw the full ten-million-resource estate as a single graph.
Clusters expose count, grouping rule, authorization scope, and drill-down.

Layout and expensive filtering run in a bounded worker/Wasm execution path;
cancel obsolete work when the query/view changes. Version layout inputs and
results so an old computation cannot replace a newer view. Node dragging,
selection, keyboard focus, labels, tooltips and hit testing have measured costs.
Avoid per-frame resource object allocation and whole-graph serialization.

Incremental view updates apply entity versions and stream cursors. Missed/gapped
streams trigger controlled snapshot refresh. Context loss releases/recreates GPU
resources and preserves navigation state. Unsupported GPU/context policies retain
a usable table/detail experience with an explicit graph capability message.

## User-visible state transitions

Initial loading -> useful bounded snapshot -> live updates. A transient network
failure retains the last displayed state with a clear last-updated/connection
status; recovery resumes or refreshes by cursor. An authorization failure clears
inaccessible data/caches and returns the user to an allowed view. A stale graph
projection displays its lag and never inherits a fresh SQL timestamp.

No-data, filtered-empty, denied, unsupported, stale, incomplete, deleted, and
unresolved are distinct states. Partial graph expansion identifies its limit and
offers filters; it does not imply no additional dependencies. Annotation revision
conflicts preserve the user's pending edit and show the current stored revision.

## Performance and accessibility acceptance

On the declared reference client/network and 5,000-node/20,000-edge bounded
view: initial useful result within 2 seconds p95, pan/zoom frame time at or below
16.7 ms p95, and input response within 100 ms p95. Measure CPU, GPU, transfer,
layout, allocation and memory separately; browser frame instrumentation cannot
be replaced by server query timing. Use fixed camera/interaction scripts and
representative high-degree graphs with labels and selection enabled.

Virtualized tables cannot create one DOM row per estate resource. Keyboard
navigation, accessible names, focus restoration, contrast, reduced motion and
screen-reader alternatives are required. The graph has an equivalent bounded
relationship list/table. Test supported current Chrome/Chromium, Firefox, and
Safari versions when available; declare the actual tested version matrix and
capability fallback rather than assuming WebGPU or identical drivers.

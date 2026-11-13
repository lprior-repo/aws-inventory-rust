# Normative requirements register

SHALL is mandatory for the replacement release. This register specifies behavior;
it does not certify implementation. User decisions and engineering qualification
profiles are distinguished in `decisions.md`. Each row maps to a scenario family
in `validation-gates.md`; the machine traceability register preserves that mapping.

| ID | EARS requirement | Acceptance family |
| --- | --- | --- |
| R001 | The system SHALL run as a self-hosted enterprise platform with isolated workspaces and multiple source registrations. | A01 |
| R002 | When a noncommercial AWS partition is registered, the system SHALL reject it with an explicit launch-scope outcome. | A01 |
| R003 | The system SHALL qualify a 3,000-account reference workload and characterize a 10,000-account stress workload without claiming untested capacity. | A22 |
| R004 | The system SHALL distinguish the declared inventory universe, provider limitations, product gaps, disabled opt-ins, and observed account permissions. | A02 |
| R005 | When a supported valid change hint is accepted, the system SHALL target refreshed or authoritatively confirmed unchanged state query-visible within five seconds; unresolved/failure outcomes SHALL be visible misses, not freshness successes. | A11 |
| R006 | While enabled, every supported expected scope SHALL complete successful reconciliation at least hourly; incomplete/denied/late scopes SHALL remain visible as breaches. | A12 |
| R007 | When a scope fails, the system SHALL retain its last known inventory with visible age/coverage while healthy scopes continue updating. | A03 |
| R008 | When absence is confirmed under the declared policy, the system SHALL mark deletion and preserve history; partial or denied enumeration SHALL NOT imply deletion. | A08 |
| R009 | The system SHALL collect configuration metadata and SHALL NOT retrieve application contents or secret values for inventory. | A17 |
| R010 | The system SHALL keep enterprise annotations distinct from immutable provider observations and SHALL NOT mutate AWS through CMDB editing. | A15 |
| R011 | When discovering an Organization, the system SHALL enumerate all pages and record account/OU membership with complete-list evidence. | A01 |
| R012 | When onboarding an account, the system SHALL verify caller account, partition, role, and enabled-region evidence before accepting inventory. | A01 |
| R013 | The system SHALL support explicitly configured existing roles, refreshable credentials, and standalone account registrations without storing session credentials. | A01 |
| R014 | When an account is removed or inaccessible, the system SHALL represent visibility loss separately from resource deletion. | A03 |
| R015 | The system SHALL produce reviewable least-privilege bootstrap plans and keep their application separate from inventory execution. | A01 |
| R016 | The system SHALL expose permission prerequisites, subscription requirements, region availability, source lag, and bootstrap drift per account. | A02 |
| R017 | The system SHALL execute global resources once per authoritative endpoint scope and preserve real resource ownership and region semantics. | A04 |
| R018 | When a role can list but cannot describe or tag a resource, the system SHALL retain partial evidence and explicit unavailable fields. | A03 |
| R019 | The system SHALL reuse SDK clients/credentials within validated account/region/lifetime boundaries and bound refresh work. | A10 |
| R020 | When region/account discovery is partial, the system SHALL NOT publish complete downstream coverage. | A03 |
| R021 | The system SHALL maintain the versioned AWS/provider/Steampipe/CloudQuery inventory-universe and compatibility matrices. | A02 |
| R022 | The system SHALL classify each SDK operation with evidence before treating it as a configuration inventory capability. | A02 |
| R023 | Each collector SHALL declare identity, scope, parent traversal, fields, tags, relationships, permissions, quotas, costs, and lifecycle semantics. | A02 |
| R024 | Each collector SHALL handle empty nonterminal pages, repeated tokens, invalid tokens, page limits, and parent failures explicitly. | A05 |
| R025 | The system SHALL preserve unavailable, missing, explicit null, absent, and redacted field states separately. | A07 |
| R026 | When provider/SDK/competitor catalogs change, the system SHALL report coverage drift without silently upgrading support claims. | A02 |
| R027 | The replacement release SHALL contain zero unexplained or product-unimplemented rows in its accepted launch universe. | A02 |
| R028 | The system SHALL publish resource, field, tag, relationship, event, and real-AWS qualification coverage separately. | A02 |
| R029 | The system SHALL record external-seed requirements for nonenumerable APIs and explicit opt-ins for chargeable/operational feeds. | A02 |
| R030 | The system SHALL retain collector/schema/policy versions with observations and compatibility views. | A19 |
| R031 | The system SHALL persist bounded collector-segment and refresh work independently of whole-account scan invocations. | A06 |
| R032 | When a worker claims work, the system SHALL issue a monotonically fenced lease using one conditional durable transaction. | A06 |
| R033 | When ownership expires, stale workers SHALL NOT commit accepted pages, checkpoints, or terminal outcomes. | A06 |
| R034 | When a worker restarts, the system SHALL resume validated checkpoints or begin a separately evidenced epoch under the collector's restart policy. | A05 |
| R035 | The system SHALL coordinate actual shared AWS quota domains across hosts and workspaces observing the same account. | A10 |
| R036 | Every explicit provider attempt SHALL consume the applicable rate/concurrency budget, including retries. | A10 |
| R037 | When ownership transfers, the system SHALL retain cooldown/rate state and bound already-dispatched request overlap. | A10 |
| R038 | The system SHALL use one classified provider retry owner with finite attempt/time budgets and validated server retry hints. | A10 |
| R039 | The scheduler SHALL prevent hot accounts, event bursts, and backfill from starving hourly reconciliation. | A12 |
| R040 | The system SHALL bound tasks, channels, pending work, payloads, memory, and connections and expose saturation. | A13 |
| R041 | When change events arrive, the system SHALL authenticate source binding and persist receipts/intents before acknowledgement. | A11 |
| R042 | When events duplicate or reorder, the system SHALL coalesce refresh work while retaining causal receipt linkage and preventing state regression. | A07 |
| R043 | When an event mapping is unknown or poisonous, the system SHALL quarantine it with bounded retry and expose a coverage/freshness problem. | A11 |
| R044 | The system SHALL reconcile hourly independently of event availability or delivery guarantees. | A12 |
| R045 | When cancellation or lease loss occurs, the system SHALL stop intake, cancel, drain, and finalize with an explicit bounded outcome. | A06 |
| R046 | Successful page acceptance SHALL atomically persist observations, page identity, checkpoint, coverage effects, and outbox intent. | A05 |
| R047 | Successful receipt acceptance SHALL atomically persist the receipt and refresh intent before upstream acknowledgement. | A11 |
| R048 | When a page/commit is replayed, unique idempotency constraints SHALL prevent duplicate logical state and detect conflicting payloads. | A05 |
| R049 | When commit outcome is ambiguous, the system SHALL resolve by idempotency lookup before retrying publication. | A06 |
| R050 | The system SHALL retain authoritative identifiers, unresolved references, owner evidence, aliases, and incarnation uncertainty. | A04 |
| R051 | Current-field selection SHALL account for freshness, capture time, authority, visibility, and comparable ordering, rather than fixed source priority alone. | A07 |
| R052 | Hybrid resources SHALL expose field-level provenance and distinct provider/receipt/observation timestamps. | A07 |
| R053 | Absence sweeps SHALL respect the epoch cutoff, exact population, principal visibility class, and later reobservations. | A08 |
| R054 | Confirmed deletion SHALL require two separated successful complete epochs or a reviewed equivalent resource-specific policy. | A08 |
| R055 | Relationship removal SHALL require completeness of that source's relationship population and preserve historical assertions. | A08 |
| R056 | Projectors SHALL apply idempotent entity versions and publish only contiguous/gap-accounted completeness watermarks. | A09 |
| R057 | When a graph/search sink fails, observations SHALL remain durable and bounded projection backlog/lag SHALL be visible. | A09 |
| R058 | Projection rebuilds SHALL validate in a separate namespace before atomic publication and retain rollback capability. | A09 |
| R059 | Retention SHALL verify archives and respect holds, replay horizons, tombstone safety, and audit policy before deleting evidence. | A20 |
| R060 | Storage migrations/imports SHALL preserve declared compatibility and verify provenance/counts/state; foreign/newer schemas SHALL fail explicitly. | A19 |
| R061 | The system SHALL publish a versioned HTTPS API/OpenAPI contract and typed clients with coverage/version/freshness metadata. | A14 |
| R062 | List/search APIs SHALL enforce cursor-bound authorization, stable pagination, result bounds, and query expiry semantics. | A14 |
| R063 | Resource APIs SHALL expose current state, history/as-of, provenance, lifecycle, aliases, relationships, and coverage evidence. | A14 |
| R064 | Graph APIs SHALL enforce typed authorized traversals, depth/node/edge/byte limits, and explicit truncation. | A14 |
| R065 | Query execution SHALL use parsed supported read-only SQL, parameter binding, least-privilege storage roles, and execution limits. | A16 |
| R066 | Compatibility views SHALL document differences from the accepted AWS Steampipe/CloudQuery table/field universe. | A02 |
| R067 | The system SHALL support saved views/queries, scheduled reports, snapshot diffs, and cancellation of expensive work. | A14 |
| R068 | CMDB APIs SHALL support owners, teams, services/applications, custom fields, annotations, and user-created relationships. | A15 |
| R069 | Concurrent edits SHALL condition on expected revision and return a conflict without overwriting stored or pending user data. | A15 |
| R070 | Control mutations SHALL require semantic authorization and idempotency keys; accepted intent SHALL remain distinct from completion. | A14 |
| R071 | Subscription streams SHALL carry resume cursors and require a controlled snapshot reset after gaps/expiry. | A14 |
| R072 | Exports SHALL support versioned JSON/JSONL/CSV/Parquet with immutable manifests, counts, hashes, and authorized downloads. | A18 |
| R073 | Webhook delivery SHALL use authenticated bounded retries, receiver idempotency, safe destinations, and visible dead letters. | A18 |
| R074 | Public errors SHALL use stable safe codes and correlation IDs without leaking secrets or inaccessible object existence. | A16 |
| R075 | The UI SHALL provide fleet coverage/freshness, onboarding, inventory, resource history, graph, CMDB editing, reports, and operations surfaces. | A21 |
| R076 | The UI SHALL use server-filtered virtualized tables and bounded GPU graph views rather than loading the full estate. | A21 |
| R077 | Graph rendering SHALL use WebGL2-capable baseline behavior and capability-tested optional WebGPU with a usable fallback. | A21 |
| R078 | Layout/filter computation SHALL run off the main UI path with cancellation and version checks against obsolete results. | A21 |
| R079 | View updates SHALL be incremental and preserve navigation/selection where authorization permits. | A21 |
| R080 | When networking fails, the UI SHALL show retained data age and recover by resume/reset; when authorization is revoked it SHALL clear inaccessible state. | A16 |
| R081 | UI no-data, filtered-empty, denied, unsupported, stale, incomplete, deleted, and unresolved states SHALL remain distinct. | A21 |
| R082 | The UI SHALL meet declared frame/input/initial-view budgets on the reference client with labels and interactions enabled. | A23 |
| R083 | The UI SHALL provide keyboard/accessibility support, reduced motion, and equivalent relationship tables. | A21 |
| R084 | Browser/GPU context loss SHALL recover bounded resources without silently losing the user's allowed view. | A21 |
| R085 | Browser compatibility SHALL be qualified on named versions/capabilities and real interaction paths. | A21 |
| R086 | The UI SHALL expose projection lag and SHALL NOT describe stale graph state using a fresh inventory timestamp. | A09 |
| R087 | The platform SHALL support enterprise OIDC SSO, group/role mapping, expiring service identities, and audited break-glass recovery. | A16 |
| R088 | Workspace/account/field authorization SHALL apply to rows, counts, edges, exports, streams, caches, logs, and indirect access paths. | A16 |
| R089 | Database runtime roles SHALL be non-owner/non-superuser/non-BYPASSRLS, with tested transaction-local context and default-deny policy. | A16 |
| R090 | TLS/endpoint validation, encryption at rest, key rotation, and workload identity SHALL protect service/provider/storage boundaries. | A17 |
| R091 | Sensitive fields SHALL be classified/redacted before generic storage, logging, graph projection, cache, or export. | A17 |
| R092 | User/provider text SHALL be bounded and safely rendered without executing embedded content. | A17 |
| R093 | The platform SHALL enforce session, CSRF, CSP, CORS, stream-origin, request-limit, SSRF, and export-path controls appropriate to each boundary. | A17 |
| R094 | Authorization revocation SHALL invalidate affected server access, streams, cursors, downloads, and cached access decisions within 30 seconds; the supported UI SHALL clear inaccessible cached state. | A16 |
| R095 | Inventory roles SHALL remain read-only; infrastructure bootstrap SHALL use distinct explicit administrator authority. | A01 |
| R096 | Security/audit records SHALL attribute actor, workspace, action, policy/revision, result, and correlation without secret payloads. | A17 |
| R097 | Source/policy/retention/edit/export/replay/admin changes SHALL generate protected audit evidence. | A17 |
| R098 | Secrets/session credentials SHALL NOT be committed or persisted in work payloads, artifacts, logs, or user-facing errors. | A17 |
| R099 | Tenant-like workspace isolation SHALL be tested even though launch is one self-hosted enterprise rather than public SaaS. | A16 |
| R100 | A dependency finding SHALL retain its disposition and evidence; compilation SHALL NOT substitute for security review. | A24 |
| R101 | The platform SHALL ship reproducible local/emulator deployment and an HA enterprise deployment profile with pinned dependencies/images. | A25 |
| R102 | The platform SHALL publish sizing, topology, resource ceilings, network/IAM prerequisites, and explicit startup/readiness dependencies. | A25 |
| R103 | Operators SHALL receive bounded-cardinality metrics, traces, logs and dashboards for stage latency, scope freshness, quota, backlog and errors. | A26 |
| R104 | Freshness, visibility loss, saturation, lease churn, projection lag, archive/backup lag, and credential failure SHALL have actionable alerts. | A26 |
| R105 | Accepted state SHALL satisfy the declared HA durability policy, and failover SHALL preserve fencing and contiguous projection semantics. | A27 |
| R106 | Backups/PITR and archive manifests SHALL be verified through actual restore exercises with stated RPO/RTO evidence. | A27 |
| R107 | Upgrades SHALL support compatibility windows, canary rollout, explicit migrations, backfill monitoring and rollback boundaries. | A19 |
| R108 | Disaster, disk-full, poisoned events, lost credentials, partitioned workers, quota collapse and graph failure SHALL have tested runbooks. | A27 |
| R109 | Inventory history and archive retention SHALL be configurable by workspace with safe expiry and holds. | A20 |
| R110 | Capacity planning SHALL include full-enumeration API lower bounds, enrichment fanout, event churn, history, WAL, replicas and rebuild headroom. | A22 |
| R111 | The system SHALL expose configured cost/resource ceilings and SHALL NOT bypass quotas or durability to meet a freshness target. | A13 |
| R112 | Decommissioning SHALL preserve/export authorized evidence and explicitly revoke role/event/integration access under operator control. | A25 |
| R113 | New domain logic SHALL use parsed private-field types, explicit state transitions, semantic errors, and a pure core/async shell boundary. | A24 |
| R114 | First-party Rust SHALL forbid unsafe and unchecked panic constructs and obey the installed strict source/function/parameter policies. | A24 |
| R115 | Async work SHALL have task ownership, bounded admission, cancellation/drain/finalize outcomes, and observable capability-gated effects. | A24 |
| R116 | Every behavior SHALL map to independent executable acceptance evidence against production boundaries, not implementation-shaped assertions alone. | A24 |
| R117 | Applicable proof/model/fuzz/concurrency lanes SHALL be planned/reviewed and bridged to production source; unexecuted design SHALL NOT count as proof. | A24 |
| R118 | Manual QA, adversarial Red Queen campaigns, strict architectural/Black Hat reviews and evidence audit SHALL report actual results and unresolved findings. | A24 |
| R119 | Final release evidence SHALL identify source, toolchain, dependencies, binaries, commands, exit status, workload, seeds and raw artifacts. | A24 |
| R120 | Local GPU work SHALL use the authorized OMP RTX 3090 route with skill-read evidence and parent verification; rejected candidates SHALL NOT be landed as accepted work. | A24 |
| R121 | Build/release CI SHALL check locked dependencies, SDK catalogs, schemas, formatting, warnings, behavior, migrations, dependency audit and reproducible artifacts. | A24 |
| R122 | Tests against PostgreSQL SHALL exercise its actual concurrency/locking/isolation semantics rather than substitute SQLite-only results. | A06 |
| R123 | Emulation SHALL preserve SDK protocol/signing behavior and distinguish fixture assertions from real-AWS qualification. | A25 |
| R124 | Production qualification SHALL include staged real-account validation and a multi-day continuous soak before thousand-account rollout. | A28 |
| R125 | Documentation SHALL include installation, onboarding, supported coverage, daily use, failure recovery, API/schema compatibility and safe upgrades. | A25 |
| R126 | Cold inventory, steady change processing, hourly reconciliation, API queries, rendering and cost SHALL be benchmarked separately and together under contention. | A22 |
| R127 | Benchmarks SHALL freeze coverage/fields/freshness/durability/quotas/hardware/versions before comparisons and validate state equality. | A29 |
| R128 | Comparisons SHALL include Steampipe cold/warm behavior and CloudQuery full/incremental/supported event-driven behavior where equivalent. | A29 |
| R129 | Interactive indexed lookup/filter/one-hop queries SHALL target p95 at most 250 ms and p99 at most 1 second on the declared 10-million-asset profile. | A23 |
| R130 | Accepted supported change processing SHALL target 99.9 percent within five seconds during the declared healthy steady load, with every miss visible and provider delivery measured separately. | A11 |
| R131 | The system SHALL measure hourly full reconciliation concurrently with event processing, UI/API traffic and projection work. | A12 |
| R132 | Performance evidence SHALL include min/median/variance, sufficient tail samples, resource/cost use, throttles, coverage and freshness rather than peak rows/s alone. | A29 |
| R133 | Optimizations SHALL follow measured profiles and preserve correctness, coverage, observability and durability on before/after workloads. | A29 |
| R134 | Market-leading claims SHALL name the workload, measured competitors, versions and confidence; a local stage result SHALL NOT establish a market-wide ranking. | A29 |
| R135 | A release SHALL remain a candidate until required evidence and independent reviews pass; known rejected findings SHALL NOT be silently reclassified as approval. | A24 |

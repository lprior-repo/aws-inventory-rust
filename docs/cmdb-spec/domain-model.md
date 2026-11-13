# Domain model

This is a design contract, not a claim that the types or service exist.

| Term | Meaning and boundary |
| --- | --- |
| Deployment | One self-hosted enterprise installation and its security/operational boundary |
| Workspace | An isolated inventory namespace with its own authorization, retention, source registrations, and overlays |
| Organization registration | A reviewed discovery configuration for an AWS Organization; several may belong to one workspace |
| Account target | A commercial AWS account and an explicitly configured read-only role; standalone accounts are supported |
| Credential binding | A reference to workload identity/role configuration, never persisted session credentials |
| Collector definition | Versioned resource population, API graph, field policy, permissions, quota domains, pagination, reconciliation, and evidence requirements |
| Scope | A closed enumeration population: workspace, source registration, owner/scanning account, endpoint scope, collector version, and optional parent/filter dimensions |
| Scope epoch | One enumeration generation with an immutable population definition, start cutoff, expected pages, and explicit terminal outcome |
| Observation | Immutable evidence obtained from a specific source, principal visibility class, scope epoch, and response/page |
| Resource identity | Provider namespace, commercial partition, authoritative owner, resource kind, and canonical provider identifier; aliases are separate evidence |
| Incarnation | One lifetime of a resource identity when provider IDs can be reused; distinguish uncertain identity continuity explicitly |
| Relationship | A typed, directed assertion with source/target identity or unresolved reference, provenance, validity, and confidence |
| Event hint | An untrusted notification that may trigger refresh; it is not automatically authoritative state or deletion evidence |
| Refresh intent | Durable, deduplicatable work to obtain a current observation; coalescing retains all causal event receipts |
| Current state | A deterministic projection from accepted observations, source rules, temporal evidence, and lifecycle decisions |
| Coverage | Explicit outcome for an expected scope/field/relationship, including denied, unsupported, stale, partial, and failed |
| Checkpoint | Durable page/event progress committed with the observations it describes |
| Deletion candidate | Suspected absence with explicit supporting successful scope epochs and resource-specific evidence |
| Tombstone | A confirmed lifecycle transition; retained history remains queryable under policy |
| Projection version | Monotonic application sequence for a read model; not a provider timestamp |
| Annotation | User-owned CMDB metadata layered over observed provider facts, with revisions and audit history |
| Service/application | An enterprise grouping of resources, ownership, criticality, dependencies, and custom fields |
| Quota domain | Actual shared AWS throttle population; workspace boundaries do not create independent AWS quotas |

## Aggregates and ownership

The source registry owns onboarding state and expected account populations.
The collector registry owns reviewed definitions and coverage promises. The
scope aggregate owns epoch completion and absence evidence. The resource
aggregate owns identity/alias/incarnation decisions and lifecycle projection.
The work aggregate owns lease generations and terminal outcomes. The workspace
aggregate owns authorization policy and retention. Provider observations and
enterprise annotations have separate ownership and revision streams.

An aggregate transaction cannot include an AWS request. Network effects occur
in the shell; a pure decision receives bounded facts and produces commands for
the next storage transaction or effect. Transaction retries cannot repeat an
unbounded network side effect accidentally.

## Commands and events

Commands include RegisterSource, ReconcileOrganization, BeginScopeEpoch,
AcceptPage, AcceptEventReceipt, RequestRefresh, ClaimWork, RenewLease,
FinalizeScope, ProposeAbsence, ConfirmDeletion, RebuildProjection, EditAnnotation,
ExportSnapshot, ChangePolicy, and RequestCancellation.

Events include AccountDiscovered, AccountVisibilityLost, EpochStarted,
PageAccepted, EventReceiptAccepted, RefreshScheduled, ObservationAccepted,
ScopeCompleted, ScopeIncomplete, AbsenceProposed, DeletionConfirmed,
ResourceReobserved, LeaseLost, ProjectionAdvanced, AnnotationChanged, and
RetentionApplied. Audit events are distinct from provider events.

## Identity decisions

Prefer returned authoritative ARNs when they describe resource identity. Parse
ARN components with service-specific rules; do not assume every AWS resource
has an ARN or that every ARN's account/region component is populated.
Non-ARN resources use a versioned namespaced canonical identifier tuple. Provider
identifiers and original names are retained exactly; no blanket lowercasing.

Aliases require reviewed equivalence evidence. Scanning-account inference never
establishes ownership of a shared resource. Unresolved references remain visible
and cannot become verified dependencies through a fabricated ARN. Ownership and
visibility are separate dimensions. A role that can list but cannot describe a
resource supplies partial evidence, not a complete object with empty fields.

## CMDB ownership model

Provider facts are read-only to CMDB users. Owners, teams, environments, business
services, criticality, lifecycle classifications, cost centers, links, runbooks,
and user-created relationships are versioned overlays. A tag-derived value and
an explicit enterprise override remain separately inspectable. Removing an
override reveals the observed value; it does not mutate AWS.

# Query API, integrations, and security

## API contract

Provide a versioned HTTPS API with generated OpenAPI and Rust client types.
Every workspace response includes authorization-safe coverage summary, read-model
version, freshness/lag, and truncation where applicable. Opaque cursors bind the
workspace, principal/policy scope, query digest, snapshot version, and expiry.
Changed parameters or revoked authorization invalidate cursor use.

| API group | Required operations |
| --- | --- |
| Identity/workspace | Current principal, workspace memberships, role/policy capabilities |
| Accounts/sources | Register/list/update/suspend sources, Organizations/OU tree, caller verification, onboarding and permission status |
| Coverage | Resource/field/relationship matrix, expected scopes, missed deadlines, denied/unsupported scopes and corrective guidance |
| Resources | Filter/search/paginate, detail, field provenance, history/as-of view, aliases, incarnation and lifecycle |
| Relationships | Typed bounded neighborhood, dependency paths, unresolved-reference evidence, historical edges |
| CMDB | Owners/teams/services/applications, custom fields, annotations and user edges; optimistic revision updates |
| Queries/reports | Saved parameterized queries, supported read-only SQL, scheduled reports, snapshot diffs and evidence links |
| Work/operations | Health/readiness, scheduler/backlog, cancellation, scoped reconciliation, retry/dead-letter inspection and replay |
| Delivery | Bounded snapshot exports, delivery jobs, signed authorized downloads, webhook configuration and status |
| Policy/audit | Access policy, retention/holds, source/collector policy revisions, attributable audit search |

Mutating control-plane endpoints require idempotency keys and semantic command
authorization. Edits require If-Match/expected revision; conflicts return 412
without overwriting another editor. Cancellation reports accepted intent and
eventual outcome separately. A 202 response is not completed work.

Default bounded query limits: 100 rows/page, maximum 1,000; interactive SQL
statement deadline 5 seconds; graph traversal depth at most 3 and maximum 5,000
nodes/20,000 edges/8 MiB per interactive view. Return explicit truncation and
refinement options. Longer exports/reports are asynchronous bounded jobs with
operator-configured total byte/time budgets and cancellation.

## SQL and graph interfaces

Expose curated compatibility views and a parsed, documented read-only SQL subset
through the query service. Parameter binding, database read-only transactions,
least-privilege roles, object/function restrictions, statement/row/byte limits,
and workspace enforcement all apply. Text prefix checks are not SQL security.
CTEs, functions, extensions, metadata access, timing/count leakage, and malformed
query attacks require explicit handling. A raw PostgreSQL port or unrestricted
Cypher endpoint is not exposed to normal browser users.

Publish supported joins, aggregates, JSON/tag access, timestamps, and compatibility
differences. Saved SQL remains versioned and revalidated after schema changes.
Graph requests use a typed traversal/filter contract, not arbitrary user-supplied
server query text. External read-only SQL connectivity can be a separately
qualified authenticated gateway; do not promise full wire compatibility before
that surface has its own isolation and failure evidence.

## Enterprise identity and authorization

Launch requires OIDC SSO, group-to-workspace roles, and service identities with
expiring scoped tokens. Roles: platform administrator, workspace administrator,
inventory operator, CMDB editor, analyst/viewer, and audited automation identity.
Identity provider enforces organizational MFA. SAML-only environments may use
an explicitly supported identity broker; native SAML is not silently assumed.
Provide an audited break-glass recovery procedure with short lifetime and alerts.

Authorization covers workspace/account/OU/tag or service ownership scopes and
sensitive fields. Apply it to searches, counts, relationships, exports, cached
results, logs, subscriptions, and download URLs. Object identifiers do not grant
access. Re-check authorization on use, including long-running exports/streams.
Role revocation closes streams and invalidates cached authorization within
30 seconds. New requests revalidate within that maximum policy-cache lifetime.
Protected downloads pass through live authorization; a long-lived provider
presigned URL must not bypass revocation. Already downloaded client data cannot
be recalled; the supported UI clears its inaccessible in-memory/local caches.

PostgreSQL runtime roles are non-owner, non-superuser, and have no BYPASSRLS.
Use row security/default deny where appropriate and test pool/context reset.
Separate migrations, collectors, projectors, query users, and backup credentials.
RLS alone is insufficient for sensitive-field and cross-workspace aggregate
protection. Administrators' broader rights are explicit and audited.

## Data and transport protection

TLS for browser/API, storage and service transport; validated AWS TLS and endpoint
policy; encrypted databases/object archives/backups; managed secret references
and key rotation. Workload identity and refreshable roles replace static keys.
No keys/credentials in source, job payloads, screenshots, logs, or exports.

Classify fields before generic persistence. Never collect secret values, private
keys, application contents, or executable package bodies for CMDB inventory.
Tags, names, URLs, policy text, and annotations are untrusted and may contain
sensitive data or markup: classify, bound, escape, and audit their use. Configure
CSP, CSRF/session controls, secure cookies, controlled CORS, request limits, and
WebSocket/SSE origin/authentication checks. Test SSRF via endpoints/webhooks and
path traversal via export/archive filenames.

## Integrations and audit

Ship versioned JSON/JSONL/CSV/Parquet exports, immutable manifests/hashes, signed
workspace-authorized downloads, and authenticated generic webhooks with bounded
retry/dead-letter behavior and idempotency keys. Secrets live in a secret store.
Prevent arbitrary private-network webhook access unless explicitly allowed.
Specific ServiceNow/Jira/other adapters require their own declared mapping and
qualification; the generic interface is not a claim that they already exist.

Audit login, denied access, registrations/roles, collector/policy changes, edits,
exports, replays, deletion decisions, holds, retention and administrator actions.
Bind actor, request, workspace, before/after revision, policy and result. Protect
audit integrity and retention; avoid sensitive payloads and unbounded labels.

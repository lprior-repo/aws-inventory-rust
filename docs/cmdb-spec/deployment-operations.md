# Deployment, observability, recovery, and maintenance

## Deployment profiles

Local development/QA: reproducible container composition for PostgreSQL, FalkorDB,
object-storage-compatible archive service and the existing AWS emulator, plus Rust
services and the browser application. Use isolated ports, database names, fixture
accounts and generated credentials. Preserve the existing local installation and
real credential environment. The standalone SQLite CLI remains supported.

Enterprise: Kubernetes manifests/Helm packaging with separate API/control,
collector workers and projector workloads; externally managed or qualified HA
PostgreSQL; isolated FalkorDB; object storage; OIDC; secret references and OTLP
endpoint. Document DNS, TLS, ingress, AWS egress/private endpoints, network policy,
resource requests/limits, anti-affinity, readiness and upgrade requirements.
Pin image digests and manifests; do not deploy mutable latest tags.

Dependencies start in order: verified configuration/secrets -> durable storage
and compatible schema -> control/work admission -> collectors/projectors -> API
readiness and UI. Liveness checks process health; readiness checks ability to
serve the promised mode. Graph degradation does not falsely mark inventory data
lost, and a healthy HTTP process does not prove fresh inventory.

## Proposed reference topology

For capacity qualification, start with eight collector workers at 8 vCPU/16 GiB
each; PostgreSQL primary plus two standbys at 8 vCPU/32 GiB each; graph at
16 vCPU/64 GiB; and two API/control instances at 4 vCPU/8 GiB each. This totals
112 vCPU and 304 GiB before storage/object-service overhead. It is a candidate
test budget, not a sizing claim. Publish exact disks/IOPS, network, zone placement,
versions and costs before running comparisons. Tune only with retained evidence.
Smaller deployments get a separate measured supported envelope.

Use synchronous durable replication for accepted-state single-node/zone-loss
protection in the HA profile. If the required durable quorum is unavailable,
stop acknowledging new accepted state. Failover must select a safe replicated
timeline and prevent a former primary from accepting writes. Query replica lag
remains distinct from primary health. Cross-region disaster objectives are a
separate recovery decision and cannot inherit a zero-loss HA claim.

## Observability contract

Metrics: expected/successful/denied/incomplete/overdue scopes; observation/capture
age; receipt-to-durable, durable-to-refresh, refresh-to-projection and full
receipt-to-visible latency; provider delivery age where measurable; queue age,
bytes and saturation; coalescing/dedup rates; per-quota attempt/throttle/cooldown;
credential refresh health; lease loss/renewal lag; page/record byte limits; DB
commit/lock/WAL/replica lag; graph outbox age/rebuild progress; query/stream/error
latency; archive/backup validation; and measured cost/resource consumption.

Keep metric labels bounded. Resource IDs, event IDs, arbitrary tags, tokens and
unbounded scope strings belong in authorized indexed diagnostics or sampled
traces, not metric labels. Account-level detail can be queried from the control
store; aggregate dashboards must not create a metric series for every resource.

Propagate correlation IDs from receipts/work through AWS attempts, commits,
projection and query visibility. Structured logs redact before emission. Normal
production tracing is sampled/configurable; failure evidence remains useful.
Diagnostic builds support async task inspection without claiming it is enabled
or free in every timed production run.

Alerts include five-second event misses, approaching/missed hourly deadlines,
discovery shrinkage/visibility loss, quota collapse, admission/storage saturation,
lease churn, credential expiry, projection gaps/lag, failing backup/archive
verification, clock uncertainty, unexpected secret detection, and authorization
anomalies. Each alert names severity, affected scope, owner, safe diagnosis,
backpressure state and runbook. No page for every duplicate retry.

## Runbooks and disaster recovery

Required runbooks: new Organization/account; lost/expired role; disabled region;
partial IAM access; quota storm; poisoned events/dead letters; stuck/expired work;
database disk full/failover/corruption; graph outage/rebuild; archive verification
failure; secret/key rotation; unauthorized access; schema rollback; disaster
restore; and controlled decommissioning.

Backups use verified base backups plus continuous WAL/PITR where supported.
Encrypt and restrict them separately from the application. Periodically restore
into an isolated environment, validate schema and counts/state digests, replay
outbox/checkpoints, then measure RPO/RTO. A successful upload is not restore proof.
Archive replay must preserve identities, tombstones, policies and schema versions.

During a declared incident, retain accepted data, stop unsafe admission and show
degraded status. Recovery orders authoritative storage/fencing first, work and
projectors next, queries/UI afterward. Catch-up scheduling preserves fairness and
does not overrun AWS quotas. Never reset checkpoints wholesale as an undocumented
repair. Reprocessing old events must not resurrect expired/deleted incarnations.

## Upgrades and release operations

Build reproducibly from a pinned Rust toolchain, locked crates, recorded SDK
release and image digests. Review dependency/catalog drift regularly and qualify
updates before rollout. No blanket claim that every linked SDK method was run.
Generate SBOM/provenance, audit findings and signatures for distributable artifacts.

Use canary workers and compatible collector definitions; migrate schemas through
expand/backfill/verify/contract. Record the point after which rollback requires
restore or forward repair. Preserve old exports/CLI compatibility through explicit
adapters. Disable new admission during unsafe maintenance; preserve accepted work.

Decommission by stopping intake, draining, exporting/validating authorized data,
retaining required audit/holds, revoking integrations/roles/event paths explicitly,
then removing deployment resources under an administrator's reviewed plan.

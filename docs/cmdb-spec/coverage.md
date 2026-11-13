# AWS coverage and replacement boundary

## Launch boundary

Self-hosted enterprise; commercial AWS only; multiple AWS Organizations and
standalone accounts; continuously running collectors; read-only configuration
inventory, resource relationships, enterprise annotations, history, SQL/export,
and a browser UI. Noncommercial partitions must return an explicit unsupported
launch-scope result rather than accidentally using commercial endpoints.

"Everything" means a published and reviewed inventory universe, with visible
gaps. It does not mean that linking every SDK implements every AWS collector.
It does not authorize reading application records, object contents, secrets,
Lambda source packages, log-event bodies, or private keys. Configuration of
those services is in scope. Operational/billing/health feeds and chargeable APIs
require explicit classification and enablement; their exclusion remains visible.

## Coverage universe

Construct a versioned union of: AWS service configuration APIs and resource
schemas; commercial regional/global availability; the pinned Steampipe AWS table
catalog; and the pinned CloudQuery AWS table/event support catalogs. Review
parent/child traversal and fields, not just table names. New upstream resources
enter a coverage-drift report and cannot silently become "supported".

Classify every SDK operation as configuration inventory read, discovery/credential
read, application-data read excluded, mutation excluded, operational/billing
opt-in, or unavailable commercial. Name/verb heuristics can propose a class but
cannot approve it. Some reads are paid, sensitive, stateful, or nonenumerable.
An API with no list mechanism requires an explicit external-seed limitation.

Every inventory resource row must declare:

- Stable type/kind/identity/incarnation and provider aliases.
- Scope, owner versus scanning account, global/home-region behavior, parent inputs.
- Enumeration/detail/tag/policy/relationship operations and pagination contracts.
- Required/optional fields; nullable versus unavailable versus redacted semantics.
- IAM actions, resource restrictions, role visibility, subscription prerequisites.
- Actual quota dimensions, retry classification, SDK behavior, and cost class.
- Sensitive/write-only exclusions, field limits, and normalization/provenance.
- Event mappings, targeted-read capability, hourly full reconciliation strategy.
- Absence/NotFound/relationship deletion policy and eventual-consistency caveats.
- Compatibility views, schema version, fixtures, real-AWS evidence, and known gaps.

## Readiness states

Unreviewed -> Classified -> Planned -> Implemented -> EmulatorVerified ->
AwsVerified -> ProductionQualified. A separate ProviderUnsupported,
PrerequisiteMissing, PermissionDenied, ProductNotImplemented, or OptInDisabled
reason explains a gap. ProductNotImplemented must never be renamed unsupported
to improve the completion percentage. Show required-field and relationship
coverage separately from resource enumeration coverage.

The replacement release gate requires zero unexplained rows and zero
ProductNotImplemented rows within the accepted launch universe. Each exclusion
must cite a genuine scope/provider restriction and remain visible in reports.
Completion of early milestones does not satisfy this replacement gate.

## Current seed registers

`catalogs/current-collector-descriptors.jsonl`: 450 descriptors, comprising 32
native kinds and 418 Cloud Control types (397 roots, 21 parent-dependent).
This is not 450 distinct normalized kinds; adapters overlap. Catalog descriptors
do not establish field completeness, event support, or production qualification.

`catalogs/sdk-service-review-register.jsonl`: 437 pinned service SDKs.
`catalogs/sdk-operation-review-register.jsonl`: 19,643 SDK operation modules,
including mutations and data-plane operations. All begin unreviewed in the new
universe-classification process. These are static source-audit inputs, not a list
of implemented inventory operations. The upstream pin and hashes accompany them.

Existing native gaps include RDS/Aurora and expanded IAM objects; tag coverage
for DynamoDB/Lambda/ELBv2; and additional network, security, operational, and
service-specific relationships. Generic Cloud Control breadth does not close
native enrichment or relationship gaps automatically.

## Required family coverage

The universe review must cover compute/containers/serverless; networking/DNS/CDN;
storage/backup/replication; relational/NoSQL/cache/analytics databases; IAM and
Identity Center configuration; Organizations/RAM/account configuration; security,
KMS/certificates/secrets metadata; messaging/integration/workflows; API gateways;
monitoring/logging configuration; deployment/CloudFormation/developer services;
search/streaming/data transfer; AI/ML configuration; and remaining commercial
service families discovered from the SDK/provider/competitor union. Specialty,
legacy, global, parent-only, and subscription-gated types are explicitly reviewed.

## Compatibility

Provide documented AWS inventory SQL views and versioned JSON/JSONL/CSV/Parquet
exports, plus a compatibility mapping from each accepted competitor AWS table
and field. Preserve null/unavailable distinctions and timestamp semantics.
Record unsupported SQL functions, extensions, and provider-specific table shapes.
This is AWS CMDB replacement, not a claim to implement other cloud providers or
the entire third-party plugin ecosystem.

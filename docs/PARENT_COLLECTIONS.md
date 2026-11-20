# Parent-dependent Cloud Control collection

> Status: Baseline being replaced. Parent-dependent Cloud Control collection is
> part of the path the new architecture does not use (V01, V04); ROADMAP M9.5
> deletes the parent workflows and their catalog, and parent-to-child bindings
> come from the API models and overlay instead (M1.4). See [VISION](VISION.md)
> and [ROADMAP](ROADMAP.md).

The v0.6 implementation adds 24 reviewed parent-dependent types to the existing
397 regional root types. The original `catalog/cloudcontrol-resources.json`
remains unchanged; `catalog/cloudcontrol-parent-resources.json` holds the new
descriptors and explicit discovery bindings. `cloud_control::catalog()` exposes
the combined 421 types, while `root_catalog()` and `parent_catalog()` expose the
separate catalogs for evidence and migration tooling.

These counts describe implemented collection plans. They do not claim live AWS
provider validation. The pinned official emulator filters stored models using
`ResourceModel`, but its generic property store does not execute each real AWS
provider's read/list handler. Successful emulator reads alone therefore do not
establish production provider behavior.

## Reviewed discovery bindings

| Parent type | Sanitized parent property | Child `ResourceModel` property | Child resource types |
|---|---|---|---|
| `AWS::EKS::Cluster` | `Name` | `ClusterName` | `Addon`, `FargateProfile`, `Nodegroup`, `AccessEntry`, `PodIdentityAssociation` |
| `AWS::ElasticLoadBalancingV2::LoadBalancer` | `LoadBalancerArn` | `LoadBalancerArn` | `Listener` |
| `AWS::ApiGateway::RestApi` | `RestApiId` | `RestApiId` | `Authorizer`, `Deployment`, `DocumentationVersion`, `GatewayResponse`, `Model`, `RequestValidator`, `Resource`, `Stage` |
| `AWS::ApiGatewayV2::Api` | `ApiId` | `ApiId` | `Authorizer`, `Deployment`, `Integration`, `Route`, `Stage` |
| `AWS::Logs::LogGroup` | `LogGroupName` | `LogGroupName` | `LogStream`, `SubscriptionFilter` |
| `AWS::AppConfig::Application` | `ApplicationId` | `ApplicationId` | `Environment`, `ConfigurationProfile` |
| `AWS::AppConfig::Application` | `ApplicationId` | `ApplicationIdentifier` | `ExperimentDefinition` |

Bindings are explicit records checked against both pinned schemas and each
child's list-handler schema. AWS documents `ResourceModel` as the request field
for these parent constraints. List results may omit properties, which is why
this collector obtains parent context from a successful `GetResource` result.
See the official [list operation guide](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html)
and [ListResources reference](https://docs.aws.amazon.com/cloudcontrolapi/latest/APIReference/API_ListResources.html).

The pinned source remains
`catalog/upstream/cloudformation-us-east-1-2026-11-12.zip`, SHA-256
`8226b9cd82f432997fe3918b134ef155245636f35ac37f8006de5abe88dd54a8`.
The generator reads this archive and the frozen root catalog without modifying
either. Reproduce and check the additional descriptors with:

```bash
cargo run --locked -p catalog-tool -- universal
cargo run --locked -p catalog-tool -- universal --check
```

## Selection and execution

A child-only selection automatically adds its exact root prerequisite. Selecting
several children of one root still enumerates that root once. Selecting a root
alone does not implicitly add children; selecting the whole catalog includes all
reviewed children.

```bash
aws-inventory scan --db inventory.sqlite --region us-east-1 \
  --service cloud-control --cloud-control \
  --resource-type AWS::EKS::Addon,AWS::EKS::FargateProfile
```

The underlying `AwsScanOptions.resource_types` accepts these exact type names.

Each root list page is validated before its bounded parent reads begin. A
successful read is sanitized using the existing write-only and sensitive-value
policy. Only reviewed string properties are copied into a small canonical JSON
model. Child lists receive that same model on every pagination request.

Parent context is streamed into child collection. The implementation retains no
complete parent inventory, and the next root page waits for the bounded work
started by the current page. Child type concurrency is two per parent and child
read concurrency is four. The existing type concurrency is four and parent read
concurrency is four. Every actual HTTP attempt also passes through the shared
executor's global concurrency, service bucket, rate admission and retry policy.

No additional SDK operation identities are introduced: the implementation uses
the existing Cloud Control `ListResources` and `GetResource` operations. The
SDK retains one attempt per call; the shared executor owns retry count and
backoff. `GetResource` receives only the provider's exact listed identifier,
never a guessed concatenation of parent and child names.

## Scope and completeness semantics

The root list and root read scopes retain their existing names. Each child list
scope is keyed by its type plus the SHA-256 of its canonical request model.
Each child read scope additionally includes the hash of the provider identifier.
No raw parent-model value is embedded in scope keys.

For each selected child type, a `cloud_control_parent_discovery` scope records
whether its prerequisite enumeration completed. A successful empty root list
produces successful empty child-discovery evidence without making child calls.
A denied root list, page limit, invalid page, or failed required parent read
produces `parent_discovery_incomplete`. Missing, empty, non-string, control
character or excessive parent context also produces an explicit failed context
scope. A parent failure does not erase children discovered under other parents.

Child lists and reads retain their own failure scopes. A completed prerequisite
scope proves only that all available parent contexts were obtained; it does not
override a child provider error or an admission cap. Overall snapshot completeness
continues to derive from every persisted scope and failure.

## Relationship and redaction contract

A detailed child response must contain every requested model property with the
same string value. A mismatched or missing required property fails the read.
This prevents an emulator or provider that ignores a filter from generating a
false parent edge. The stored child observation includes `list_resource_model`,
`parent_resource_id`, and `parent_type` for provenance.

After that independent read comparison succeeds, the collector adds a
`CHILD_OF` edge to the actual observed parent resource identity. It records the
reviewed binding evidence and target kind. Existing schema-derived architecture
edges are preserved. A list-only fallback never adds architecture edges and
cannot change its scope's failed status. An explicitly contradictory list
property is rejected rather than attached to the requested parent.

Additional redactions include EKS addon configuration values, API Gateway stage
variables, request/response templates and parameter maps, and AppConfig
validator content. Redaction occurs before observations, provider columns,
names, tags, parent context or relationships are derived.

## Bounds and cancellation

Each root group permits at most 10,000 child-model list scopes across all its
child types and at most 100,000 distinct admitted child identifiers. These are
aggregate group limits, not a new 100,000-item allowance for every parent.
Admission fails before further child requests when the corresponding limit is
reached. Existing 10,000-page, list-page, provider-JSON, token-history and
per-scope limits remain in force.

Parent models contain at most four bindings, each copied value is at most 1,024
bytes, and encoded model size is at most 8,192 bytes. The current 24 plans each
use exactly one binding. A group ledger tracks bounded hashed identities and
rejects the same child identifier across parent scopes. It also rejects two
distinct parent resources producing the same child model before a second list
can restart that scope. Locks protect only
synchronous admission state and are never held over an await.

The implementation creates no detached tasks. Cancellation is checked before
parent/child requests and after traversal, and all work remains owned by the
collector's bounded futures. Fatal channel failure propagates; ordinary provider
errors remain explicit partial-coverage records.

## Verification obligations

`cloud_control/parents/tests.rs` includes literal expectations for all 24 model
bindings, selection closure/idempotence properties, context bounds, denied and
empty discovery, parent-page limits, malformed and mismatched responses,
redaction, per-parent cursors, bounded admission, and cancellation.

The additional literal `cloudcontrol-parents.json` fixture covers two parents
for each of the six roots, with two semantic children under A and one under B
for every child type. Ninety-eight extra EKS addons bring that type to 101
resources and force pagination through the emulator's default 100-item window.
Parent B's first filtered addon page is empty and still has a continuation token.
The seed/oracle pair requires 161 detailed child observations, 161 exact
`CHILD_OF` edges, and 24 separately authored architecture edges, including the
EKS node role whose property is named `NodeRole` rather than ending in `Arn`.
The node-role mapping follows the official [Nodegroup property contract](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-eks-nodegroup.html#cfn-eks-nodegroup-noderole)
and matches the native collector's `ASSUMES_ROLE` relation.

`cloud_control/parents/wire_tests.rs` exercises the actual production SDK path
against a bounded in-process HTTP server. Two parents have independent child
pages, including empty non-terminal pages. The server validates every serialized
model and token, injects a first-request throttle, and the test compares exact
executor attempt/retry/throttle counts. The server and collector are polled in
one owned test future with a deadline; no background fixture process is left
running.

These are implementation/test descriptions, not test-run receipts. Current
command outcomes belong in the release verification artifacts. Live AWS
permissions, regional provider availability, real pagination behavior and any
official-emulator gaps remain separate evidence requirements.

## Still excluded

Two-level discovery such as ELBv2 `ListenerRule` requires a subsequent dependency
level and is not enabled by this change. Global/account/home-region constraints
also require explicit plans. ECS `TaskSet` is not inferred from a service: AWS's
list guide includes `ID` in the required context, so an independently reviewed
native workflow is preferable to guessing that an incomplete model enumerates
every task set. Other unreviewed schema-required parent models remain excluded.

# Account-global inventory

> Status: Baseline being replaced. The `global_inventory` adapter described
> here is replaced by overlay data on the wire engine in ROADMAP M5.2 (IAM) and
> M5.3 (S3, Route 53, CloudFront) and deleted in the same changes. See
> [VISION](VISION.md) and [ROADMAP](ROADMAP.md).

The native global inventory module adds seven resource kinds for S3, IAM,
Route 53, and CloudFront. Collection runs once for each resolved account scan,
outside the regional collector fan-out. It records control-plane architecture
metadata, explicit relationships, and incomplete scopes in the same collection
event model used by the regional adapters.

This module is an inventory adapter. It does not enumerate S3 object contents,
evaluate effective IAM permissions, resolve DNS over the network, or assess
whether an application actually accepts traffic.

## Integration contract

`crates/aws/src/global_inventory.rs` exports these crate-local entry points:

```rust
pub(crate) const GLOBAL_SERVICES: [&str; 4] =
    ["s3", "iam", "route53", "cloudfront"];

pub(crate) async fn collect(
    base: &Context,
    services: &BTreeSet<String>,
) -> Result<(), AwsScanError>;
```

The coordinator resolves the caller account and partition with STS, then calls
`collect` once with the selected global services. It must not call this function
from a regional loop. The function clones the supplied context, retaining its
credentials, HTTP configuration, single-attempt SDK retry configuration, shared
`ApiExecutor`, cancellation token, and output sender.

The SDK endpoint region and persisted inventory region are separate:

| Partition | Bootstrap SDK region | Persisted scope | Coverage |
| --- | --- | --- | --- |
| `aws` | `us-east-1` | `global` | All four global selectors |
| `aws-cn` | `cn-north-1` | `global` | S3, IAM, Route 53; CloudFront emits an unsupported-partition failure |
| `aws-us-gov` | `us-gov-west-1` | `global` | S3, IAM, Route 53; CloudFront emits an unsupported-partition failure |
| Other partitions | No reviewed endpoint policy | No success assertion | Explicit invalid-options error |

These endpoint choices are implemented policies, not a claim that each listed
service is enabled for every account. SDK endpoint or authorization failures
remain failed collection scopes. This increment has not been exercised against
live AWS accounts in any partition.

S3 bucket enumeration is account-global. Once the actual bucket region is known,
bucket metadata requests use a regional SDK configuration and regional quota
key, and the bucket resource stores that actual region. Its ARN remains the
global S3 bucket ARN. If location discovery fails, the listed bucket is retained
with `region = 'global'`, `region_resolved = false`, and a failed detail scope.

Native dependencies are `aws-sdk-s3`, `aws-sdk-iam`, `aws-sdk-route53`, and
`aws-sdk-cloudfront`; `percent-encoding` decodes IAM trust documents. Versions
are selected by the workspace lockfile. No workflow engine or extra retry loop
is introduced by these adapters.

## Resource kinds and catalog mapping

All rows include the shared resource fields: canonical identifier, scanning
account, region, kind, source, observation time, name, tags, and JSON properties.
The table lists exact property keys for integration with the SQL catalog.
`[]` means a JSON array, and dotted names denote nested properties.

| Native kind | Primary collection operations | Selected property keys | Relationships emitted |
| --- | --- | --- | --- |
| `aws_s3_bucket` | `ListBuckets`; conditional `HeadBucket`; ten metadata/tag reads below | `bucket_name`, `creation_date`, `bucket_region`, `region_resolved`, `details_available`, `tags_available`, `unavailable_sections`, `versioning`, `encryption`, `public_access_block`, `policy_status`, `logging`, `replication`, `website`, `ownership_controls`, `notifications` | `ASSUMES_ROLE`, `LOGS_TO`, `REDIRECTS_TO_DNS`, `ENCRYPTED_WITH`, `REPLICATES_TO`, `REPLICA_ENCRYPTED_WITH`, `NOTIFIES_TOPIC`, `NOTIFIES_QUEUE`, `NOTIFIES_FUNCTION` |
| `aws_iam_role` | `ListRoles`, `GetRole`, `ListAttachedRolePolicies` | `role_name`, `role_id`, `path`, `create_date`, `description`, `max_session_duration`, `permissions_boundary_arn`, `last_used_date`, `last_used_region`, `trust_statements[]`, `trust_policy_values_omitted`, `details_available`, `observation_detail` | `USES_PERMISSIONS_BOUNDARY`, `DECLARES_TRUST`, `ATTACHED_POLICY` |
| `aws_iam_instance_profile` | `ListInstanceProfiles`, `GetInstanceProfile` | `instance_profile_name`, `instance_profile_id`, `path`, `create_date`, `role_arns[]`, `details_available`, `observation_detail` | `HAS_ROLE` |
| `aws_iam_policy` | `ListPolicies` with `Scope=Local`, `GetPolicy` | `policy_name`, `policy_id`, `path`, `default_version_id`, `attachment_count`, `permissions_boundary_usage_count`, `is_attachable`, `description`, `create_date`, `update_date`, `policy_scope`, `policy_document_collected`, `details_available`, `observation_detail` | None from policy metadata; incoming role attachment/boundary edges retain the actual ARN |
| `aws_route53_zone` | `ListHostedZones`, `GetHostedZone`, `ListTagsForResource` | `zone_id`, `name`, `private_zone`, `comment`, `resource_record_set_count`, `name_servers[]`, `linked_service_principal`, `vpcs[]`, `details_available`, `tags_available`, `observation_detail` | `ASSOCIATED_WITH_VPC` |
| `aws_route53_record` | `ListResourceRecordSets` per hosted zone | `zone_id`, `name`, `type`, `set_identifier`, `ttl`, `records[]`, `rdata_omitted`, `rdata_count`, `alias_target`, `weight`, `region`, `failover`, `multi_value_answer`, `health_check_id`, `traffic_policy_instance_id`, `geo_location`, `cidr_routing`, `identity_model` | `IN_HOSTED_ZONE`, `ALIASES_DNS_NAME`, `USES_HEALTH_CHECK`, `DNS_TARGET` |
| `aws_cloudfront_distribution` | `ListDistributions`, `ListTagsForResource` | `distribution_id`, `domain_name`, `status`, `enabled`, `staging`, `http_version`, `is_ipv6_enabled`, `price_class`, `last_modified_time`, `aliases[]`, `origins[]`, `origin_groups[]`, `default_cache_behavior`, `cache_behaviors[]`, `viewer_certificate`, `web_acl_id`, `connection_mode`, `tags_available`, `observation_detail` | `ORIGINATES_FROM`, `USES_VPC_ORIGIN`, `USES_CERTIFICATE`, `USES_WEB_ACL`, `ASSOCIATES_LAMBDA`, `ASSOCIATES_FUNCTION`, `LOGS_TO` |

All collected timestamps are epoch milliseconds. `source` is `Direct` for these
adapters. An unresolved relationship target is a reference, not evidence that
the referenced resource was successfully inventoried.

### S3 bucket sections

`ListBuckets` uses explicit pagination, including `MaxBuckets`, and consumes
`BucketRegion` when present. If it is absent, `HeadBucket` supplies the region.
AWS documents that `HeadBucket` exposes the region even when access is denied;
the collector accepts a valid region header on HTTP 301, 400, 403, or 404 as
location evidence only. HTTP 429 and 5xx continue through the executor's retry
classification. Every bucket metadata request includes `ExpectedBucketOwner`.
Location evidence does not establish permission to read bucket metadata.
[ListBuckets](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html)
and [bucket location guidance](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLocation.html).

| Property section | Operation | Persisted content |
| --- | --- | --- |
| `versioning` | `GetBucketVersioning` | `status`, `mfa_delete` |
| `encryption` | `GetBucketEncryption` | `configured`, rules with `algorithm`, `kms_key_id`, `bucket_key_enabled` |
| `public_access_block` | `GetPublicAccessBlock` | `configured`, `block_public_acls`, `ignore_public_acls`, `block_public_policy`, `restrict_public_buckets` |
| `policy_status` | `GetBucketPolicyStatus` | `configured`, authoritative `is_public` when returned |
| `logging` | `GetBucketLogging` | `configured`, `target_bucket`; target prefixes and grants omitted |
| `replication` | `GetBucketReplication` | `configured`, `role_arn`, rules with ID, status, priority, destination bucket/account/storage class/KMS ID; filter values omitted |
| `website` | `GetBucketWebsite` | `configured`, redirect host/protocol, presence of index/error documents, routing-rule count; object keys omitted |
| `ownership_controls` | `GetBucketOwnershipControls` | `configured`, ownership-mode rules |
| `notifications` | `GetBucketNotificationConfiguration` | Topic/queue/Lambda IDs, actual ARNs, event types, EventBridge flag; filter values omitted |
| Resource `tags` | `GetBucketTagging` | Tag key/value map; availability is separate from an empty tag set |

The JSON distinction is deliberate:

```json
{"policy_status":{"configured":true,"is_public":true}}
{"policy_status":{"configured":false}}
{"policy_status":null,"unavailable_sections":["policy_status"]}
```

The first form is evidence returned by `GetBucketPolicyStatus`. The second is
used only for its modeled `NoSuchBucketPolicy` response. The third indicates
unavailable or unrequested evidence and is accompanied by a failed scope when
the attempted request failed. Other metadata sections use similarly explicit
modeled absence codes. Access denial is never translated to `configured=false`.
The actual `unavailable_sections` array can include multiple sections and tags.
See [GetBucketPolicyStatus](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicyStatus.html).

`details_available` is true only after location, all nine sections, and tags are
available. A section containing a modeled absence is available evidence. S3
objects, object versions, raw bucket policy text, ACL grants, arbitrary filter
values, and bucket contents are not collected. Account-level S3 public access
block settings are not collected in this increment. Bucket policy status and
bucket block settings do not establish effective end-to-end access.

KMS key IDs and aliases remain properties. An `ENCRYPTED_WITH` or
`REPLICA_ENCRYPTED_WITH` edge is emitted only when AWS supplies an ARN, avoiding
invented ownership or region for an alias. Replication destinations and
notification targets retain their exact reported ARNs. Logging bucket owner
resolution is explicitly unknown.

### IAM metadata and trust evidence

`ListRoles` omits role tags, permission boundaries, and last-used details, so
each listed role is enriched with `GetRole`. A failed detail read preserves the
list observation with `details_available=false` and `observation_detail=list_only`.
Instance profiles and customer-managed policies use the same list/detail
pattern. See [ListRoles](https://docs.aws.amazon.com/IAM/latest/APIReference/API_ListRoles.html)
and [GetPolicy](https://docs.aws.amazon.com/IAM/latest/APIReference/API_GetPolicy.html).

Trust documents are RFC 3986 percent-decoded; a literal `+` remains `+`.
The mapper retains statement effect, action names, principal references,
condition operator names, and the presence of `NotAction` or `NotPrincipal`.
It excludes raw policy documents and condition values. `DECLARES_TRUST` edges
describe explicit `Allow` principals and carry qualifiers plus
`effective_permission_inferred=false`. They do not claim that the principal
can successfully assume the role. Deny statements and `NotPrincipal` do not
become affirmative trust edges.

Customer-managed policies are enumerated with `Scope=Local`; AWS-managed
policies can appear as referenced attachment targets without becoming native
policy resources in this increment. Policy document versions, inline policies,
users, groups, access keys, credential reports, and authorization simulation are
outside this adapter. Tags are copied when returned by the detail response;
independent IAM tag-list operations are not performed.

### Route 53 identity and pagination

Hosted zone IDs normalize an optional `/hostedzone/` prefix and become
`arn:<partition>:route53:::hostedzone/<id>`. Record sets have no fabricated AWS
ARN: their inventory ID is a SHA-256 digest of length-framed partition, account,
zone, normalized name, record type, and optional routing identifier. A presence
component distinguishes a simple record from a routing identifier whose value
happens to equal the internal placeholder. Weighted, latency, and failover
records sharing a DNS name/type therefore remain distinct.

Pagination preserves **NextRecordName, NextRecordType, and NextRecordIdentifier**
together. The serialized cursor is bounded and validated before reuse. A
truncated response without the required next name/type fails the scope instead
of silently reporting completion. AWS documents that changes during traversal
can yield a changing view; this collector is not a transactional DNS snapshot.
[ListResourceRecordSets](https://docs.aws.amazon.com/Route53/latest/APIReference/API_ListResourceRecordSets.html).

TXT, SPF, and unknown-type RDATA are omitted with explicit counts. A, AAAA,
CNAME, NS, PTR, MX, SRV, SOA, and CAA records retain bounded routing values.
A null MX record (`0 .`) is retained without a DNS destination edge, following
[RFC 7505 section 3](https://www.rfc-editor.org/rfc/rfc7505.html#section-3).
IP targets are scoped to partition/account/zone to avoid conflating private
addresses across unrelated networks. Alias targets remain DNS references;
the collector does not manufacture an ELB ARN from a hostname. Zone VPC
references have an explicit `owner_resolution=assumed_scanning_account` because
the API's VPC association does not establish the owner account in that field.

### CloudFront architecture components

The adapter collects conventional distribution summaries and tags. Origins,
origin groups, and cache behaviors remain nested components on the distribution
record. Their keys are suitable for SQL `json_each` expansion without adding
misleading standalone resource identities. See
[ListDistributions](https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_ListDistributions.html)
and [DistributionSummary](https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_DistributionSummary.html).

Origins retain IDs, DNS names, origin type, protocol/port/timeouts, access
identity/control IDs, origin-shield configuration, and reported VPC origin
references. Origin paths and custom header values are omitted. Behaviors retain
path pattern, target-origin ID, whether that ID exists in the returned
configuration, viewer protocol, allowed methods, policy IDs, TTLs, Lambda and
CloudFront Function associations, and real-time log ARN. Forwarded cookie/query
configuration is omitted. Behavior TTL fields are legacy values as returned by
the API; cache-policy bodies are not fetched, so those fields are not a complete
calculation of effective caching behavior.

An origin matching a reviewed S3 DNS naming pattern points to a bucket ARN with
explicit DNS-pattern evidence and unknown owner. Other origin domains remain
`dns:` references. Certificate, WAF, Lambda, Function, and log relationships use
reported ARNs. Lambda version qualifiers are preserved. VPC origin references
use a namespaced inventory identity, retaining the reported owner or explicitly
marking the scanning account assumption. VPC origin resources, distribution
tenants, functions, certificates, WAF ACLs, and cache policy bodies are not
independently fetched by this module.

## Execution, bounds, and partial results

Every SDK `.send()` is inside `Context::execute(service, "read", ...)`. The
shared executor owns admission, pacing, attempt timeouts, retry-after handling,
bounded retries, and cancellation. These collectors do not call SDK paginators
or create detached tasks. Continuation pages are sequential within a scope;
independent resources and service scopes use bounded streams.

| Boundary | Implementation |
| --- | --- |
| Global service fan-out | At most four services per account |
| IAM family fan-out | Three independent list streams |
| Child detail fan-out | Four children per list stream |
| S3 section fan-out | Four section requests per active bucket |
| Service list page sizes | S3/IAM at most 1,000; Route 53 zones/CloudFront at most 100; Route 53 records at most 300 |
| Scope page/child limits | Shared configurable page limit, hard maximum 10,000 pages; at most 100,000 listed children per scope |
| List identity ledger | SHA-256 digests only, at most 100,000 per scope; duplicate IDs within/across pages fail explicitly |
| Continuation ledger | Shared duplicate-token and byte limits from `common::paginate` |
| Route 53 cursor | At most 2 KiB; bounded name/type; routing identifier at most 128 Unicode characters and 512 UTF-8 bytes |
| Component mapping | At most 4,096 per checked component group; distribution components use an aggregate budget |
| Relationship inputs | At most 4,096 per resource; merged duplicate evidence retains the shared 1,024-observation limit |
| Trust policy | At most 64 KiB encoded text, 128 statements, 256 principals per principal category per statement |
| DNS RDATA | At most 512 values per record; shared identifier/property/page size validation still applies |

These are work-admission and mapper bounds, not a claim that the SDK cannot
allocate an oversized HTTP response before the adapter inspects it. The process
also retains one current response page per active list scope. Shared executor
limits and the bounded event channel determine actual request concurrency and
backpressure across accounts and regions. No throughput benchmark is claimed
for these adapters, and SDK or service request quotas are not discovered
automatically by this module.

Each bucket, role, profile, policy, zone, and distribution detail operation has
a child scope. A normal detail failure emits failure evidence before any
list-only fallback. It does not cancel valid sibling children. S3 section and
tag failures are separate subscopes; available metadata remains usable. Invalid
record metadata produces a record-specific failure while valid mapped siblings
can remain on the same page. List identity, cursor, page-size, and aggregate
payload violations fail the parent traversal; no completeness claim is made
for the discarded page. Cancellation and a closed consumer propagate without
manufacturing a successful fallback.

List scopes emit checkpoints only after their children have finished. The
storage layer's failure-sticky scope state is required: a later fallback page
must not erase a preceding failure in that scope. Retry does not guarantee a
transactionally consistent snapshot. The report must consult scope state when
deciding whether a missing resource was observed to be absent.

## Permissions and request planning

These selectors perform read operations. The caller still needs permission for
each selected operation, and an account or resource may deny individual reads.

| Selector | IAM actions used |
| --- | --- |
| `s3` | `s3:ListAllMyBuckets`; conditional `s3:ListBucket` for successful `HeadBucket`; `s3:GetBucketVersioning`, `s3:GetEncryptionConfiguration`, `s3:GetBucketPublicAccessBlock`, `s3:GetBucketPolicyStatus`, `s3:GetBucketLogging`, `s3:GetReplicationConfiguration`, `s3:GetBucketWebsite`, `s3:GetBucketOwnershipControls`, `s3:GetBucketNotification`, `s3:GetBucketTagging` |
| `iam` | `iam:ListRoles`, `iam:GetRole`, `iam:ListAttachedRolePolicies`, `iam:ListInstanceProfiles`, `iam:GetInstanceProfile`, `iam:ListPolicies`, `iam:GetPolicy` |
| `route53` | `route53:ListHostedZones`, `route53:GetHostedZone`, `route53:ListTagsForResource`, `route53:ListResourceRecordSets` |
| `cloudfront` | `cloudfront:ListDistributions`, `cloudfront:ListTagsForResource` |

S3 API names sometimes differ from IAM action names; the permissions above use
the IAM action spelling. See the
[S3 API/action mapping](https://docs.aws.amazon.com/AmazonS3/latest/userguide/using-with-s3-policy-actions.html).

Before retries, a bucket normally costs ten metadata/tag calls beyond list
pagination, plus a region-discovery call when needed. An IAM role costs a detail
call plus attached-policy pages; a profile or customer policy costs a detail
call. A hosted zone costs a detail call, a tag call after successful detail,
and its record pages. A distribution costs a tag call beyond list pagination.
This request cost is deliberate: catalog presence and metadata completeness
are recorded separately, and authorization failure is retained as evidence.

## Verification and remaining limits

The offline test suite exercises canonical identities, partition selection,
composite cursors, duplicate list identities, malformed sibling isolation,
failure-before-fallback ordering, cancellation, closed consumers, IAM trust
redaction, S3 absent-versus-denied evidence, tag completeness, DNS payload
omission, explicit ARN references, and CloudFront origin/header redaction.
These tests build synthetic AWS SDK model values. They do not require or call
AWS credentials, endpoints, DNS resolvers, or metadata services.

Run the module's tests with:

```sh
cargo test -p inventory-aws global_inventory --locked -j1
```

Live-account permission coverage, eventual-consistency behavior, service
endpoint variations, and large-account throughput require a separate authorized
integration campaign. The asynchronous Rust review skill's bundled reference
files were absent in the supplied environment; its main rules and the available
Holzman references informed implementation, but a complete skill-specific formal
review or performance gate is not claimed by this document.

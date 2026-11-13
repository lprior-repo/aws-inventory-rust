# Primary sources and attribution

Research and SDK snapshot date: November 12, 2026. The SDK catalog records its
exact immutable revision, registry checksums and check timestamp separately.

## Design influences

| Project | Relevant idea | Implementation relationship |
|---|---|---|
| [awsmap](https://github.com/TocConsulting/awsmap) | Broad account/region discovery and resource inventory | Design reference; its full collectors are not ported here |
| [Cartography](https://github.com/cartography-cncf/cartography) | Resource relationships as a queryable graph | Design reference; current generic graph is not its entire Neo4j schema |
| [Steampipe AWS plugin](https://github.com/turbot/steampipe-plugin-aws) | Familiar resource table/column concepts | Selected names and independently implemented SQLite views; no full schema or PostgreSQL compatibility claim |
| [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust) | Typed service clients and standard credential providers | Direct dependency; all published service crates pinned in catalog |
| [SQLx](https://github.com/transact-rs/sqlx) | Async SQLite connection/transaction/query layer | Actual inventory and checkpoint storage dependency |
| [FalkorDB](https://github.com/FalkorDB/FalkorDB) | Redis-protocol Cypher graph database | External projection target |

This repository's new implementation is licensed under MIT; see LICENSE.
It does not bundle the upstream awsmap/Cartography/Steampipe implementations or
their documentation as a claimed complete port. The Steampipe repository
distinguishes Apache-2.0 source code from CC BY-NC-ND documentation; original
explanations are used here. Any future copied/derived upstream source needs
the applicable original notices and license terms retained.

FalkorDB server licensing and client-library licensing are separate. The source
archive does not redistribute a graph server binary. Third-party Rust packages
remain governed by the licenses in their own pinned distributions.

## AWS behavior used by the design

- [Official service SDK index](https://awslabs.github.io/aws-sdk-rust/)
- [November 11, 2026 SDK release](https://github.com/awslabs/aws-sdk-rust/releases/tag/release-2026-11-11)
- [Rust SDK retry configuration](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/retries.html)
- [AWS retry behavior and x-amz-retry-after units](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html)
- [HTTP Retry-After semantics, RFC 9110 section 10.2.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.3)
- [Rust SDK timeouts](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/timeouts.html)
- [Rust SDK client best practices](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/best-practices.html)
- [Rust SDK pagination](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/paginating.html)
- [EC2 API throttling](https://docs.aws.amazon.com/ec2/latest/devguide/ec2-api-throttling.html)
- [ECS action buckets](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/request-throttling.html)
- [Lambda quotas](https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-limits.html)
- [Lambda ListFunctions field limitations](https://docs.aws.amazon.com/lambda/latest/api/API_ListFunctions.html)
- [DynamoDB constraints](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Constraints.html)
- [Config advanced-query scope and limitations](https://docs.aws.amazon.com/config/latest/developerguide/querying-AWS-resources.html)
- [Config aggregate query API](https://docs.aws.amazon.com/config/latest/APIReference/API_SelectAggregateResourceConfig.html)
- [Resource Explorer ListResources](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html)
- [Resource Explorer supported types and type ambiguity](https://docs.aws.amazon.com/resource-explorer/latest/userguide/supported-resource-types.html)
- [ELBv2 resource ARN shapes](https://docs.aws.amazon.com/service-authorization/latest/reference/list_elbv2.html)
- [Classic ELB resource ARN shape](https://docs.aws.amazon.com/service-authorization/latest/reference/list_elb.html)
- [AWS Config supported resource types](https://docs.aws.amazon.com/config/latest/developerguide/resource-config-reference.html)
- [AWS Config load-balancer property schema](https://github.com/awslabs/aws-config-resource-schema/blob/master/config/properties/resource-types/AWS::ElasticLoadBalancingV2::LoadBalancer.properties.json)
- [Cloud Control resource-type support and handler requirements](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-types.html)
- [Cloud Control resource discovery and required parent models](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html)
- [Cloud Control ListResources; MaxResults is reserved](https://docs.aws.amazon.com/cloudcontrolapi/latest/APIReference/API_ListResources.html)
- [Cloud Control GetResource](https://docs.aws.amazon.com/cloudcontrolapi/latest/APIReference/API_GetResource.html)
- [Cloud Control authorization actions](https://docs.aws.amazon.com/service-authorization/latest/reference/list_cloudcontrol.html)
- [Cloud Control security model](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/security.html)
- [Shared VPC owner and participant responsibilities](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-share-limitations.html)
- [EC2 DescribeRouteTables](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRouteTables.html)
- [ELBv2 DescribeLoadBalancers](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_DescribeLoadBalancers.html)
- [ELBv2 DescribeTargetHealth](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_DescribeTargetHealth.html)

`NETWORKING.md`, `LOAD_BALANCING.md` and `SCHEDULER.md` include the focused
operation references for the second implementation increment. The downloaded,
pinned Rust SDK models are the concrete field/accessor source used by the adapters.

Resource Explorer explicitly documents that ListResources with MaxResults=1000 omits its
continuation token. The adapter uses at most 100 and follows tokens. Config
advanced queries cannot enumerate resources excluded from recording merely
because an aggregator can see some metadata for them.

Cloud Control is a different ListResources API: its MaxResults is reserved, so
this adapter omits that field. The 200-type catalog is derived from retained
official CloudFormation schema bytes, with per-schema checksums and acquisition
metadata in `catalog/upstream/cloudcontrol-upstream.json`. `COLLECTION_PLAN.md`
records the exact selection rules, omitted global/parent-scoped types, readable
fields, redaction and permission boundaries. The schema archive establishes
handler eligibility at the captured region and time, not successful live
collection of every type or universal regional support.

Published default rates are evidence for policy design, not a guarantee that
all accounts/actions share that quota or have no competing traffic. Explicit
quota-family mappings and verified account ceilings are separate from the
SDK's retry defaults.

## Storage and graph references

- [SQLx 0.9 SQLite](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/index.html)
- [SQLx connection worker semantics](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/struct.SqliteConnection.html)
- [SQLx native progress handler](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/struct.LockedSqliteHandle.html)
- [SQLite WAL](https://www.sqlite.org/wal.html)
- [SQLite 3.51.3 release notes](https://sqlite.org/releaselog/3_51_3.html)
- [FalkorDB GRAPH.QUERY](https://docs.falkordb.com/commands/graph.query)
- [FalkorDB indexing](https://docs.falkordb.com/cypher/indexing)
- [Cargo feature semantics](https://doc.rust-lang.org/cargo/reference/features.html)
- [Cargo fetch](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)
- [Rust's selective mostly-unused compilation hint](https://blog.rust-lang.org/inside-rust/2025/07/15/call-for-testing-hint-mostly-unused/)
- [AWS Smithy adoption of that hint](https://github.com/smithy-lang/smithy-rs/pull/4243)

Measurements in VERIFICATION.md are generated from this repository's tests and
local synthetic harness. They are not performance claims from upstream projects.

## Retry and durable-workflow research

- [Official Rust SDK Retry Behavior 2.1 announcement](https://github.com/awslabs/aws-sdk-rust/discussions/1431)
- [Temporal Rust SDK general availability, September 4, 2026](https://temporal.io/changelog/rust-sdk-generally-available)
- [Temporal Rust standalone activities](https://docs.temporal.io/develop/rust/activities/standalone-activities-quickstart)
- [Restate service retry and pause configuration](https://docs.restate.dev/services/configuration)

`SDK_RETRY_AUDIT.md` records the pinned Rust runtime source behind the retry
decision, and `RETRY_AND_WORKFLOWS.md` separates request retries from durable
job ownership. No external workflow service is a runtime dependency of this
version.

## Global inventory and account discovery in 0.4

The field, endpoint, omission and permission contracts are documented in
[GLOBAL_INVENTORY.md](GLOBAL_INVENTORY.md) and [DISCOVERY.md](DISCOVERY.md).
Operation signatures are also checked against the pinned official SDK sources.

- [S3 ListBuckets](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html)
- [S3 HeadBucket and region headers](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html)
- [S3 GetBucketPolicyStatus](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicyStatus.html)
- [IAM ListRoles](https://docs.aws.amazon.com/IAM/latest/APIReference/API_ListRoles.html)
- [IAM GetPolicy](https://docs.aws.amazon.com/IAM/latest/APIReference/API_GetPolicy.html)
- [Route 53 composite record pagination](https://docs.aws.amazon.com/Route53/latest/APIReference/API_ListResourceRecordSets.html)
- [CloudFront ListDistributions](https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_ListDistributions.html)
- [CloudFront DistributionSummary](https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_DistributionSummary.html)
- [Organizations ListAccounts and State](https://docs.aws.amazon.com/organizations/latest/APIReference/API_ListAccounts.html)
- [Resource Explorer ListResources](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html)
- [RFC 7505, null MX records](https://www.rfc-editor.org/rfc/rfc7505)

Independent review found a valid null-MX edge case. A `0 .` record carries
no-mail configuration evidence, so the mapper retains the record and emits no
fabricated DNS-target edge. A regression test checks that behavior.

## AWS Emulator integration and protocol evidence

The emulator target is the full upstream source checkout at commit
`f515581569b98a9dcbd2e686c12f912fdce51bba`, paired with the official Linux amd64
binary **1.0.336**. [upstream.json](../scripts/emulator/upstream.json) retains the
package URL, checksum, publishing-key fingerprint and source pin. The installer
verifies the detached package signature and expected signer; the runner checks
the binary's reported version and commit.

The retained [package proof](../artifacts/emulator-package-verification-v04.json)
contains the matching checksum and exact machine-readable GPG signer evidence.
The [checkout proof](../artifacts/emulator-upstream-checkout-v04.json) records a
clean, non-shallow source tree with 30,585 history commits. The final ordinary
Rust builds have separate [binary/source hashes and build receipts](../artifacts/plain-binaries-v04-final.json);
the emulator source and binary are unchanged upstream artifacts.

| Primary source | Contract used |
| --- | --- |
| [Pinned AWS Emulator source](https://github.com/aws/aws-emulator/tree/f515581569b98a9dcbd2e686c12f912fdce51bba) | Concrete operation implementations, native-store integrations and unimplemented paths |
| [Official Linux setup and signature verification](https://docs.aws.amazon.com/emulator/latest/userguide/getting-started-linux.html) | Package installation and verification workflow |
| [Official SDK connection instructions](https://docs.aws.amazon.com/emulator/latest/userguide/point-sdk-to-emulator.html) | Generated credentials, explicit HTTPS endpoint, Rust `SSL_CERT_FILE` and path-style S3 access |
| [Official scope and limitations](https://docs.aws.amazon.com/emulator/latest/userguide/what-is-service.html) | In-memory state and limits on production, IAM, isolation and load-testing claims |
| [Official fidelity guidance](https://docs.aws.amazon.com/emulator/latest/userguide/fidelity.html) | Distinguishing tested behavior from service completeness |
| [AWS Emulator evaluation suite](https://github.com/aws/aws-emulator-evaluation-suite) | Additional test-to-operation evidence, separate from this application's execution |
| [Resource Explorer GetIndex](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetIndex.html) | String representation of `CreatedAt` and `LastUpdatedAt` |
| [Resource Explorer GetView](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetView.html) | String representation of `View.LastUpdatedAt` |
| [Resource Explorer ListResources](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html) | String resource/property timestamps, continuation semantics and `Search` authorization action |
| [Official Smithy StaticReplayClient](https://docs.rs/aws-smithy-http-client/latest/aws_smithy_http_client/test_util/struct.StaticReplayClient.html) | In-memory response replay plus explicit request verification for isolated SDK protocol tests |

[EMULATOR_TESTING.md](EMULATOR_TESTING.md) records the reproducible commands,
verified TLS on both proxy hops, generated emulator credentials, exact fixture
boundaries and retained run evidence. The operation inventory has **72 explicit
SDK operations plus the SDK-owned STS AssumeRole boundary**. The **200 Cloud
Control types** share its ListResources/GetResource API pair; they do not imply
200 independent service implementations or full native-provider fidelity.

The pinned source has unconditional HTTP 501 stubs for Organizations
[ListAccounts](https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-organizations-awsorganizationsv20161128/src/ops/list_accounts.rs)
and
[ListChildren](https://github.com/aws/aws-emulator/blob/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-organizations-awsorganizationsv20161128/src/ops/list_children.rs).
The observed Resource Explorer responses contain numeric timestamps where the
official API, upstream Smithy model and pinned Rust SDK require strings.
[The signed-TLS reproduction](../artifacts/emulator-resource-explorer-wire-check.json)
retains the actual wire-type and Rust failure evidence. Source analysis and the
exact serializer/model links are in the test guide. Acknowledged gaps remain
failed operation outcomes in the harness.

The [final rebuilt-binary run](../artifacts/emulator-v04/summary.json) recorded
**1,276 requests**, of which **686** were Rust collector/probe requests and **590**
were excluded Python setup/startup traffic. All **73 operation identities** were
attempted: 64 had production HTTP success, four had verified separate Rust probe
success, two retained HTTP 501 outcomes, and three retained SDK decode failures.
The 64 count includes the separately flagged AssumeRole provider boundary.
All **1,069 harness checks** passed: 53 global fixture assertions, 984 regional
fixture assertions and 32 orchestration/storage/coverage checks. Those counts
include repeated per-resource assertions and are not distinct API-test counts.

The native scan persisted 150 resources across all 32 native kinds and 340
relationships. Cloud Control attempted all 200 types; 197 received successful
list and read responses, and the three unregistered namespaces remained failed
scopes. The final ledger contains 93 noninitial-cursor requests across 29
operations, separate from the 41 offline pagination mappings. These are results
from the seeded local emulator, not live AWS coverage, quota or performance
claims. The [earlier expanded run](../artifacts/emulator-v04-expanded/summary.json)
is retained separately from the final rebuilt-binary evidence.

Two offline test suites supplement, but do not replace, integration results:

- **19 recorder/receipt tests** independently serialize all 72 operation mappings
  and 41 pagination mappings, enforce evidence/endpoint boundaries, and prevent
  HTTP success or Python fixture traffic from being mistaken for a verified Rust
  probe. [Passing log](../artifacts/emulator-recorder-receipts.log).
- **Six Rust SDK protocol tests** accept three modeled ISO 8601 responses and
  independently reject all five numeric timestamp locations across GetIndex,
  GetView and ListResources. They use constructed in-memory fixtures, the
  unchanged pinned SDK and its official replay transport, with no AWS or emulator
  connection. [Test source and procedure](../tests/sdk-wire/README.md),
  [passing tests](../artifacts/sdk-wire-tests-v04.log),
  [strict Clippy](../artifacts/sdk-wire-clippy-v04.log).

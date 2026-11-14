# Implemented AWS call matrix

> Status: Historical typed-adapter call matrix. The SDK-derived inventories
> were removed in M1.7; the legacy adapters remain for the M2.6 differential
> proof. Counts and outcomes below describe their recorded baseline and are not
> a current inventory or execution claim. Links into `artifacts/` point to an
> ignored local directory and do not resolve in a clone. See [VISION](VISION.md)
> and [ROADMAP](ROADMAP.md).

This document preserves the historical collector, discovery and credential-provider call matrix. The frozen baseline recorded **79 distinct explicit SDK operations at 80 production `.send()` callsites**. A reviewed additive overlay recorded **10 further operations at 11 callsites**: five KMS operations at five callsites and five Step Functions operations at six callsites. The baseline plus overlay recorded **89 distinct operations at 91 production callsites across 19 services** and **39 native resource kinds**, backed by 74 operations. The overlay data was removed in M1.7; these counts are historical and must not be treated as the current SDK inventory. Typed adapters remain the M2.6 differential proof target.

The SQS and SNS adapter rows bind native-phase evidence from
[`artifacts/emulator-sns-native/summary.json`](../artifacts/emulator-sns-native/summary.json).
That baseline completed every native collector and regional assertion; its only
failed checks were the separately owned parent Cloud Control `AWS::EKS::Addon`
page-cap assertions (100 addons at page size one exceeds the 100-page budget).
Historical v0.4 counts and outcomes below remain scoped to that run.

The catalog preserves the source and executable hashes of the earlier endpoint
audit. Later property tests and the explicit retry-origin correction are
reconciled in
[`aws-operation-source-reconciliation-v04.json`](../artifacts/aws-operation-source-reconciliation-v04.json).
Every explicit SDK operation and its recorded callsite position remains the
same. That artifact records current source hashes and a fresh emulator baseline
with 1,071 passing checks and the same 68-success/five-failure operation split;
it does not relabel the historical binary as the updated one or claim that the
mutation campaigns are complete.

This document contains the source snapshot and linked results from the final signed emulator run. The source inventory alone does not establish execution success. The observed-results section and each JSON entry preserve the distinction between collector requests, direct branch probes, fixture writes, provider credential traffic, and failed operations.

| Code area | Distinct explicit operations |
| --- | ---: |
| Regional native collectors | 49 |
| Account/global native collectors | 25 |
| Identity and enabled-region bootstrap | 2 |
| Organizations and source discovery | 9 |
| Config and Resource Explorer inventory | 2 |
| Cloud Control list/get adapter | 2 |
| **Total** | **89** |

STS `GetCallerIdentity` has two explicit callsites, one for scan bootstrap and one for discovery. SDK-owned `AssumeRole` is an additional credential-provider boundary, outside the 89-operation count. The ten additive rows are source-bound to exact callsites, file hashes and pinned SDK operation modules; the overlay reports `pending_execution` and carries no emulator outcome. The historical matrix and its receipts remain byte-identical.

## How to match requests

Use each JSON entry's `boto3_service` plus `api_operation` to match the recorder. `id` uses the Rust SDK identifier plus the snake_case client method. `elasticloadbalancingv2` maps to Boto3 `elbv2`, and `resourceexplorer2` maps to `resource-explorer-2`.

Each entry includes request scope, parent inputs, predecessor operations, constants, pagination fields, IAM actions, source file/function/lines, source hashes, exact wire metadata, and relevant offline test suites. REST paths alone are insufficient: Query services share `POST /` and require the `Action` parameter; AWS JSON services require `X-Amz-Target`. S3 endpoint resolution can move the bucket between host and path.

API operation names are verified against the pinned Rust SDK operation modules in `Cargo.lock`. HTTP metadata comes from the installed Boto3/Botocore service models, with Botocore version and per-operation model hashes recorded. Some current Rust SDKs use shared schema serializers, so a grep for literal `.method(...)` calls would miss valid operations.

## Explicit operations

The tables below use API names for readability. Snake_case SDK methods, detailed pagination, and exact source locations are in the JSON. A predecessor is a collection dependency, not an IAM dependency. Explicit source configuration can bypass discovery, as described below.

### cloudcontrol

SDK `aws-sdk-cloudcontrol 1.122.0`; recorder/Boto3 name `cloudcontrol`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `GetResource` / `get_resource` | `cloudformation:GetResource` | account-region; TypeName, Identifier (exact returned identifier) |
| `ListResources` / `list_resources` | `cloudformation:ListResources` | account-region; TypeName |

### cloudfront

SDK `aws-sdk-cloudfront 1.143.0`; recorder/Boto3 name `cloudfront`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListDistributions` / `list_distributions` | `cloudfront:ListDistributions` | account-global; No resource parent |
| `ListTagsForResource` / `list_tags_for_resource` | `cloudfront:ListTagsForResource` | account-global; Resource (distribution ARN) |

### config

SDK `aws-sdk-config 1.134.0`; recorder/Boto3 name `config`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `DescribeConfigurationAggregators` / `describe_configuration_aggregators` | `config:DescribeConfigurationAggregators` | account-region; No resource parent |
| `SelectAggregateResourceConfig` / `select_aggregate_resource_config` | `config:SelectAggregateResourceConfig` | account-region; ConfigurationAggregatorName |

### dynamodb

SDK `aws-sdk-dynamodb 1.139.0`; recorder/Boto3 name `dynamodb`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `DescribeTable` / `describe_table` | `dynamodb:DescribeTable` | account-region; TableName |
| `ListTables` / `list_tables` | `dynamodb:ListTables` | account-region; No resource parent |

### ec2

SDK `aws-sdk-ec2 1.278.0`; recorder/Boto3 name `ec2`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `DescribeInstances` / `describe_instances` | `ec2:DescribeInstances` | account-region; No resource parent |
| `DescribeInternetGateways` / `describe_internet_gateways` | `ec2:DescribeInternetGateways` | account-region; No resource parent |
| `DescribeNatGateways` / `describe_nat_gateways` | `ec2:DescribeNatGateways` | account-region; No resource parent |
| `DescribeNetworkAcls` / `describe_network_acls` | `ec2:DescribeNetworkAcls` | account-region; No resource parent |
| `DescribeNetworkInterfaces` / `describe_network_interfaces` | `ec2:DescribeNetworkInterfaces` | account-region; No resource parent |
| `DescribeRegions` / `describe_regions` | `ec2:DescribeRegions` | bootstrap region; No resource parent |
| `DescribeRouteTables` / `describe_route_tables` | `ec2:DescribeRouteTables` | account-region; No resource parent |
| `DescribeSecurityGroups` / `describe_security_groups` | `ec2:DescribeSecurityGroups` | account-region; No resource parent |
| `DescribeSubnets` / `describe_subnets` | `ec2:DescribeSubnets` | account-region; No resource parent |
| `DescribeVolumes` / `describe_volumes` | `ec2:DescribeVolumes` | account-region; No resource parent |
| `DescribeVpcEndpoints` / `describe_vpc_endpoints` | `ec2:DescribeVpcEndpoints` | account-region; No resource parent |
| `DescribeVpcPeeringConnections` / `describe_vpc_peering_connections` | `ec2:DescribeVpcPeeringConnections` | account-region; No resource parent |
| `DescribeVpcs` / `describe_vpcs` | `ec2:DescribeVpcs` | account-region; No resource parent |

### ecs

SDK `aws-sdk-ecs 1.159.0`; recorder/Boto3 name `ecs`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `DescribeClusters` / `describe_clusters` | `ecs:DescribeClusters` | account-region; Clusters[] |
| `DescribeServices` / `describe_services` | `ecs:DescribeServices` | account-region; Cluster, Services[] |
| `DescribeTaskDefinition` / `describe_task_definition` | `ecs:DescribeTaskDefinition` | account-region; TaskDefinition (exact ARN and revision) |
| `DescribeTasks` / `describe_tasks` | `ecs:DescribeTasks` | account-region; Cluster, Tasks[] |
| `ListClusters` / `list_clusters` | `ecs:ListClusters` | account-region; No resource parent |
| `ListServices` / `list_services` | `ecs:ListServices` | account-region; Cluster |
| `ListTaskDefinitions` / `list_task_definitions` | `ecs:ListTaskDefinitions` | account-region; No resource parent |
| `ListTasks` / `list_tasks` | `ecs:ListTasks` | account-region; Cluster |

### eks

SDK `aws-sdk-eks 1.162.0`; recorder/Boto3 name `eks`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `DescribeCluster` / `describe_cluster` | `eks:DescribeCluster` | account-region; Name |
| `DescribeNodegroup` / `describe_nodegroup` | `eks:DescribeNodegroup` | account-region; ClusterName, NodegroupName |
| `ListClusters` / `list_clusters` | `eks:ListClusters` | account-region; No resource parent |
| `ListNodegroups` / `list_nodegroups` | `eks:ListNodegroups` | account-region; ClusterName |

### elasticloadbalancingv2

SDK `aws-sdk-elasticloadbalancingv2 1.136.0`; recorder/Boto3 name `elbv2`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `DescribeListeners` / `describe_listeners` | `elasticloadbalancing:DescribeListeners` | account-region; LoadBalancerArn |
| `DescribeLoadBalancers` / `describe_load_balancers` | `elasticloadbalancing:DescribeLoadBalancers` | account-region; No resource parent |
| `DescribeRules` / `describe_rules` | `elasticloadbalancing:DescribeRules` | account-region; ListenerArn |
| `DescribeTargetGroups` / `describe_target_groups` | `elasticloadbalancing:DescribeTargetGroups` | account-region; No resource parent |
| `DescribeTargetHealth` / `describe_target_health` | `elasticloadbalancing:DescribeTargetHealth` | account-region; TargetGroupArn |

### iam

SDK `aws-sdk-iam 1.135.0`; recorder/Boto3 name `iam`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `GetInstanceProfile` / `get_instance_profile` | `iam:GetInstanceProfile` | account-global; InstanceProfileName |
| `GetPolicy` / `get_policy` | `iam:GetPolicy` | account-global; PolicyArn |
| `GetRole` / `get_role` | `iam:GetRole` | account-global; RoleName |
| `ListAttachedRolePolicies` / `list_attached_role_policies` | `iam:ListAttachedRolePolicies` | account-global; RoleName |
| `ListInstanceProfiles` / `list_instance_profiles` | `iam:ListInstanceProfiles` | account-global; No resource parent |
| `ListPolicies` / `list_policies` | `iam:ListPolicies` | account-global; No resource parent |
| `ListRoles` / `list_roles` | `iam:ListRoles` | account-global; No resource parent |

### kms

SDK `aws-sdk-kms 1.131.0`; recorder/Boto3 name `kms`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListKeys` / `list_keys` | `kms:ListKeys` | account-region; Limit (clamped to 1..=1000), Marker |
| `DescribeKey` / `describe_key` | `kms:DescribeKey` | account-region; KeyId (exact listed key ARN) |
| `GetKeyRotationStatus` / `get_key_rotation_status` | `kms:GetKeyRotationStatus` | account-region; KeyId (exact key ARN), customer-managed keys only |
| `ListResourceTags` / `list_resource_tags` | `kms:ListResourceTags` | account-region; KeyId (exact key ARN), Limit 50 |
| `ListAliases` / `list_aliases` | `kms:ListAliases` | account-region; Limit (clamped to 1..=1000), Marker |

### lambda

SDK `aws-sdk-lambda 1.158.0`; recorder/Boto3 name `lambda`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListFunctions` / `list_functions` | `lambda:ListFunctions` | account-region; No resource parent |

### organizations

SDK `aws-sdk-organizations 1.139.0`; recorder/Boto3 name `organizations`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListAccounts` / `list_accounts` | `organizations:ListAccounts` | organization-global; No resource parent |
| `ListChildren` / `list_children` | `organizations:ListChildren` | organization-global; ParentId (root or OU) |
| `ListOrganizationalUnitsForParent` / `list_organizational_units_for_parent` | `organizations:ListOrganizationalUnitsForParent` | organization-global; ParentId (root or OU) |
| `ListRoots` / `list_roots` | `organizations:ListRoots` | organization-global; No resource parent |

### resourceexplorer2

SDK `aws-sdk-resourceexplorer2 1.126.0`; recorder/Boto3 name `resource-explorer-2`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `GetDefaultView` / `get_default_view` | `resource-explorer-2:GetDefaultView` | account-region; No resource parent |
| `GetIndex` / `get_index` | `resource-explorer-2:GetIndex` | account-region; No resource parent |
| `GetView` / `get_view` | `resource-explorer-2:GetView` | account-region; ViewArn |
| `ListResources` / `list_resources` | `resource-explorer-2:Search` | account-region; ViewArn |
| `ListViews` / `list_views` | `resource-explorer-2:ListViews` | account-region; No resource parent |

### route53

SDK `aws-sdk-route53 1.134.0`; recorder/Boto3 name `route53`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `GetHostedZone` / `get_hosted_zone` | `route53:GetHostedZone` | account-global; Id |
| `ListHostedZones` / `list_hosted_zones` | `route53:ListHostedZones` | account-global; No resource parent |
| `ListResourceRecordSets` / `list_resource_record_sets` | `route53:ListResourceRecordSets` | account-global; HostedZoneId |
| `ListTagsForResource` / `list_tags_for_resource` | `route53:ListTagsForResource` | account-global; ResourceId (hosted-zone ID) |

### s3

SDK `aws-sdk-s3 1.159.0`; recorder/Boto3 name `s3`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `GetBucketEncryption` / `get_bucket_encryption` | `s3:GetEncryptionConfiguration` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketLogging` / `get_bucket_logging` | `s3:GetBucketLogging` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketNotificationConfiguration` / `get_bucket_notification_configuration` | `s3:GetBucketNotification` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketOwnershipControls` / `get_bucket_ownership_controls` | `s3:GetBucketOwnershipControls` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketPolicyStatus` / `get_bucket_policy_status` | `s3:GetBucketPolicyStatus` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketReplication` / `get_bucket_replication` | `s3:GetReplicationConfiguration` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketTagging` / `get_bucket_tagging` | `s3:GetBucketTagging` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketVersioning` / `get_bucket_versioning` | `s3:GetBucketVersioning` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetBucketWebsite` / `get_bucket_website` | `s3:GetBucketWebsite` | bucket actual region; Bucket, ExpectedBucketOwner |
| `GetPublicAccessBlock` / `get_public_access_block` | `s3:GetBucketPublicAccessBlock` | bucket actual region; Bucket, ExpectedBucketOwner |
| `HeadBucket` / `head_bucket` | `s3:ListBucket` | bucket endpoint region discovery; Bucket, ExpectedBucketOwner |
| `ListBuckets` / `list_buckets` | `s3:ListAllMyBuckets` | account-global; No resource parent |

### sns

SDK `aws-sdk-sns 1.123.0`; recorder/Boto3 name `sns`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListTopics` / `list_topics` | `sns:ListTopics` | account-region; NextToken |
| `GetTopicAttributes` / `get_topic_attributes` | `sns:GetTopicAttributes` | account-region; TopicArn (after `sns.list_topics`) |
| `ListTagsForResource` / `list_tags_for_resource` | `sns:ListTagsForResource` | account-region; ResourceArn (topic ARN) |
| `ListSubscriptions` / `list_subscriptions` | `sns:ListSubscriptions` | account-region; NextToken; rows filtered by subscription ARN scope |

### sqs

SDK `aws-sdk-sqs 1.121.0`; recorder/Boto3 name `sqs`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListQueues` / `list_queues` | `sqs:ListQueues` | account-region; MaxResults (clamped to 1..=1000), NextToken |
| `GetQueueAttributes` / `get_queue_attributes` | `sqs:GetQueueAttributes` | account-region; QueueUrl; one batched call with nine explicit attribute names |
| `ListQueueTags` / `list_queue_tags` | `sqs:ListQueueTags` | account-region; QueueUrl |

### stepfunctions

SDK `aws-sdk-sfn 1.127.0`; recorder/Boto3 name `stepfunctions`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `ListStateMachines` / `list_state_machines` | `states:ListStateMachines` | account-region; maxResults (clamped to 1..=1000), nextToken |
| `DescribeStateMachine` / `describe_state_machine` | `states:DescribeStateMachine` | account-region; stateMachineArn (exact listed ARN) |
| `ListTagsForResource` / `list_tags_for_resource` | `states:ListTagsForResource` | account-region; resourceArn (exact state machine or activity ARN) |
| `ListActivities` / `list_activities` | `states:ListActivities` | account-region; maxResults (clamped to 1..=1000), nextToken |
| `DescribeActivity` / `describe_activity` | `states:DescribeActivity` | account-region; activityArn (exact listed ARN) |

### sts

SDK `aws-sdk-sts 1.126.0`; recorder/Boto3 name `sts`.

| API / SDK method | IAM action | Scope and parent inputs |
| --- | --- | --- |
| `GetCallerIdentity` / `get_caller_identity` | `sts:GetCallerIdentity` (allow permission not required) | credential identity at bootstrap endpoint; No resource parent |

## Required fixture and branch coverage

### Identity, regions, and account ownership

Both bootstrap paths must obtain and validate `GetCallerIdentity`. Global inventory runs once per account, outside the regional fan-out. `DescribeRegions(AllRegions=false)` runs when regions are not explicit and regional inventory or automatic source discovery needs them. The emulator harness uses one explicit region to keep the test bounded; a separately labelled Rust SDK probe can exercise `DescribeRegions` without claiming the collector's discovery branch executed.

Every explicit service operation passes through `ApiExecutor` with SDK `max_attempts=1`. Rate buckets distinguish account, partition, region, service, and operation group; global services use global scope and S3 detail requests use the actual bucket region. Bootstrap identity uses the separate `identity:{bootstrap_region}:sts` bucket. Retry, cancellation, scoped errors, and backpressure are covered by offline engine/common tests; emulator success alone does not prove quota behavior.

### Regional native parents

- EC2's twelve native describe operations inventory unfiltered resource classes. Fixtures must include VPCs, subnets, security groups, interfaces, volumes, routes, gateways, endpoints, ACLs, peering, and instances to validate architecture edges.
- DynamoDB requires nonempty `ListTables` to reach `DescribeTable`. Lambda's implemented inventory uses `ListFunctions`; it does not call `GetFunction` or separate tag APIs.
- ECS cluster pages lead to `DescribeClusters`, per-cluster services, and separate RUNNING and STOPPED task scopes. `DescribeServices` batches at most 10 ARNs; task descriptions batch at most 100. ACTIVE task definitions are listed independently and described by exact ARN and revision, with `TAGS` requested.
- EKS lists node groups even when cluster detail fails. A nonempty node-group list is necessary to reach `DescribeNodegroup`.
- ELBv2 listener rules are requested only for **application** load balancers. A load balancer without listeners does not cover `DescribeRules`. Target groups are listed independently and need registered targets to exercise membership edges and `DescribeTargetHealth` result mapping.
- KMS needs at least two customer-managed keys and one alias. `ListKeys` must page or complete, `DescribeKey` must confirm the listed key identity, and `ListAliases` must return the alias ARN and target key ID. If the pinned emulator answers `GetKeyRotationStatus` or `ListResourceTags` with HTTP 501, the fixture assertion must require the recorded child-scope failure and retained key with `rotation_collected`/`tags_collected` false rather than invented rotation or tag state.

### Global native fixtures

S3 needs a configured bucket and a bucket with absent optional configuration. The rich fixture should exercise versioning, encryption, bucket public-access-block flags, policy status, logging target, replication role/destination, website, ownership controls, notification targets, and tags. Bucket requests send `ExpectedBucketOwner`. No object contents or raw bucket policy are collected.

**`HeadBucket` is conditional.** It executes only when `ListBuckets` omits `BucketRegion`; current emulator responses may supply that field. A direct Rust branch probe should use `seed_global`'s `fixtures.s3.rich_bucket`. Preserve and label that probe separately. Do not strip real response fields and count the altered response as emulator compatibility evidence.

Modeled S3 absence has separate semantics from AccessDenied and unavailable data. For example, `NoSuchBucketPolicy` is `configured=false`, while a successful `GetBucketPolicyStatus` provides the authoritative `is_public` value. Tests must inspect both configured and absent fixtures. Merely observing a 404 is insufficient unless its service code matches the collector's absence handling.

IAM needs roles, profiles containing roles, and customer-managed policies. At least two attached policies exercise `ListAttachedRolePolicies` pagination. The policy list sends `Scope=Local` and `OnlyAttached=false`. `GetRole` supplies fields omitted from list responses; `GetPolicy` collects metadata only. Trust conditions are not stored verbatim.

Route53 needs nonempty zones and record sets, including a routing identifier so the composite name/type/identifier cursor is tested. Tags are only requested after successful `GetHostedZone`. Records are still attempted after a zone-detail failure. Useful records include ordinary addresses, aliases, weighted records, TXT payload omission, and the valid null MX `0 .`. TXT/SPF values and arbitrary unrecognized record payloads are omitted from inventory; no health-check API is implemented.

CloudFront needs full distribution summaries and tags. Origins and default/ordered cache behaviors come from `ListDistributions`, not `GetDistribution`. Fixtures should contain both an S3 origin and a custom DNS origin. The collector omits origin paths and custom-header values. CloudFront collection is explicitly unsupported outside the commercial `aws` partition, so a loopback test in `us-east-1` does not prove other-partition behavior.

### Organizations and source discovery

`discover --organization` reaches `ListAccounts`; adding `--hierarchy` reaches roots, OU traversal, and `ListChildren(ChildType=ACCOUNT)`. A nested OU and a moved member account are necessary to prove hierarchy links. Hierarchy traversal's recursive dependency is recorded explicitly in the JSON.

Config discovery lists aggregators. The inventory adapter uses `SelectAggregateResourceConfig` against an explicitly selected or uniquely discovered aggregator. Tests need nonempty JSON rows inside `Results`, not only a successful empty response. The query and its exact projection are in the manifest.

Resource Explorer discovery requires an ACTIVE index before it lists views. `GetDefaultView` runs after complete view-list pagination. The selected default must occur in that candidate list; without a default, a sole candidate may be selected. `GetView` verifies the chosen ARN before inventory. An explicitly supplied view can bypass these discovery operations. Seed an index, at least two views, a default association, and resources visible through that view to exercise all five implemented operations.

### Cloud Control provider dimension

The 200 enabled resource types use only two explicit SDK operations, `ListResources` and `GetResource`. Count the operation surface and the provider surface separately. The matrix includes the catalog hash and count. For each type, evidence should distinguish successful nonempty enumeration/detail, successful empty enumeration, unsupported/denied/other errors, and an operation never reached because no identifier was listed.

List requests omit reserved `MaxResults`. Detail requests preserve exact composite identifiers, including `|`. No service role ARN is supplied. Permissions required by each provider are additional to the two Cloud Control API IAM actions; provider-internal service requests are not Rust collector callsites.

## Credential-provider boundary

`discovery::assume_role_config` configures `aws_config::sts::AssumeRoleProvider`, optional external ID, session name, and bounded duration. Its `AssumeRole` call is SDK-owned, lazy, and can recur during credential refresh. The shared identity cache owns that network boundary; it is **not accounted by `ApiExecutor`**. The provider uses the configured single-attempt SDK retry policy, and provider construction is bounded and cancellation-aware.

The integration harness must prove that credential-provider traffic also stays on its loopback endpoint, and label provider `AssumeRole` separately. Do not print returned access keys, secret keys, session tokens, Authorization headers, or signed request bodies.

Other default-chain providers can use SSO/OIDC, web identity, instance metadata, container credentials, or a configured process depending on the caller's environment. Those environment-dependent provider protocols are not implemented collector operations. The harness isolates its AWS configuration and uses generated emulator credentials so they cannot silently become external test traffic.

## IAM action exceptions and sources

API method names are not a reliable IAM policy generator. Relevant exceptions include S3 `ListBuckets` → `s3:ListAllMyBuckets`, `HeadBucket` → `s3:ListBucket`, `GetBucketEncryption` → `s3:GetEncryptionConfiguration`, `GetPublicAccessBlock` → `s3:GetBucketPublicAccessBlock`, `GetBucketReplication` → `s3:GetReplicationConfiguration`, and `GetBucketNotificationConfiguration` → `s3:GetBucketNotification`.

Resource Explorer `ListResources` is authorized by **`resource-explorer-2:Search`**, as stated by its API and service authorization documentation. Cloud Control actions use the **`cloudformation`** IAM prefix. STS `GetCallerIdentity` does not require an allow permission. These mappings identify actions; they are not a complete deployment policy or proof of effective authorization.

Primary references:

- [S3 API operation and policy action mapping](https://docs.aws.amazon.com/AmazonS3/latest/userguide/using-with-s3-policy-actions.html)
- [Resource Explorer ListResources API](https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_ListResources.html)
- [Resource Explorer service authorization actions](https://docs.aws.amazon.com/service-authorization/latest/reference/list_awsresourceexplorer.html)
- [Cloud Control identity-based policy examples](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/security_iam_id-based-policy-examples.html)
- [STS GetCallerIdentity](https://docs.aws.amazon.com/STS/latest/APIReference/API_GetCallerIdentity.html)

## Evidence requirements

A useful run report retains service, API operation, phase, attempt count, HTTP status distribution, pagination-request count, and the Cloud Control type name where applicable. It should never retain signed headers, credentials, full request/response bodies, or raw sensitive configuration. For success claims, join transport evidence to SQLite scope completion, nonempty resource rows, and relevant edge/property assertions.

Use these distinct outcomes:

| Outcome | What it establishes |
| --- | --- |
| Collector success with result assertions | Actual collector request, SDK decoding, and asserted output behavior |
| Collector request returned error | Transport path reached; classify error and resulting scoped failure |
| Direct Rust branch probe | SDK/endpoint compatibility for a conditional operation; collector branch remains separate |
| Python seed/probe | Fixture or server setup only; never collector coverage |
| Provider request | Credential acquisition/refresh boundary, separate from explicit collector calls |
| Not observed | No coverage claim, even if the SDK method is compiled |

Request counts alone cannot prove correct pagination, completeness, retries, regional isolation, or IAM enforcement. Page-size-one fixtures help force continuations where the actual service supports that size; EC2 describe APIs clamp to their service-specific minimums instead.

## Offline tests and extraction checks

`related_offline_test_suites` records exact test names in the current AWS source tree and operation entries link the relevant suites. These are mapper, pagination, source-selection, and error-behavior tests, not endpoint tests. Some simple collectors rely on shared tests and integration coverage rather than a dedicated source-file test module.

The extraction counted every production empty-argument `.send()` in `crates/aws/src`, excluding test sections and channel `.send(event)` calls. It resolved each client operation, checked parent branches manually, verified the 39 native catalog rows against their operation names, and verified all API names against their pinned SDKs. Source hashes and line numbers capture a snapshot; refresh this matrix when a collector, SDK version, branch, or catalog changes. `builder_line` starts the SDK receiver expression, `operation_line` identifies the SDK method, and `send_line` identifies its `.send()`. Provider locations identify configuration because the SDK owns its network call. The final refresh rechecked a one-to-one mapping between all production sends and the explicit operation records of that snapshot, verified the pinned SDK versions, and linked each observed outcome to frozen evidence.

## Observed emulator run: final v0.4

The final [summary](../artifacts/emulator-v04/summary.json) and [request ledger](../artifacts/emulator-v04/requests.jsonl) contain **1,276 recorded requests: 686 from Rust collectors or explicit Rust SDK probes, and 590 excluded fixture/startup requests**. The Rust requests match exactly the 72 implemented explicit operations plus the separately identified `AssumeRole` provider boundary. All 73 boundaries were attempted; emulator compatibility is **partial**.

| Observed outcome | Boundaries | Meaning |
| --- | ---: | --- |
| Explicit collector/discovery success | 63 | Actual production SDK call with linked command/data evidence |
| SDK credential-provider success | 1 | Actual `AssumeRole` request and a completed assumed-role inventory scan |
| Successful direct Rust SDK probe | 4 | `DescribeRegions`, `HeadBucket`, `GetDefaultView`, and `ListViews`; conditional collector paths remain separate |
| Emulator endpoint not implemented | 2 | Organizations `ListAccounts` and `ListChildren` return HTTP 501 |
| SDK rejected an HTTP 200 response | 3 | Resource Explorer `GetIndex`, `GetView`, and `ListResources` fail typed decoding |
| **Total attempted** | **73** | **68 successful boundaries and five failures** |

The raw report combines the first two rows as 64 `collector_success` outcomes and explicitly flags the provider entry. The [Rust probe receipt](../artifacts/emulator-v04/branch-probes.json) supplies typed and semantic success assertions for the four successful probe operations; a recorded HTTP 200 alone cannot establish probe success. The same receipt preserves the three Resource Explorer error chains. The emulator emits numeric timestamps where the pinned Rust SDK expects date-time strings; those failures are not converted to passes. See [emulator testing](EMULATOR_TESTING.md) for the independent wire diagnosis and separately labelled offline protocol fixtures.

The [native result](../artifacts/emulator-v04/native.json) is complete: **212 successful accounted API attempts, 150 resources, 340 relationships, and 131 completed scopes**. The [SQLite evidence](../artifacts/emulator-v04/database-evidence.json), [53 global assertions](../artifacts/emulator-v04/global-assertions.json), and [984 regional assertions](../artifacts/emulator-v04/regional-assertions.json) validate actual collected properties, identities, edges, and omission rules. Together with 32 harness checks, the report has **1,069 passing checks and zero failed checks**. Some checks explicitly validate declared failure outcomes; this count does not mean every AWS operation succeeded.

All 200 catalog types received a Rust Cloud Control list request; **197 types have successful detail reads**. `AWS::S3Express::DirectoryBucket`, `AWS::S3Tables::TableBucket`, and `AWS::Scheduler::ScheduleGroup` remain unsupported provider types. Cloud Control and source-discovery scans retain their partial status and scoped failures. The request ledger records **93 noninitial-cursor requests across 29 operations**; these observations do not establish live AWS quotas, authorization, regional isolation, or every possible continuation branch.

The JSON `emulator_run` section records SHA-256 hashes for the final command/data artifacts, source helpers, harness, lockfile, and the ordinary Rust binaries used for the final run. The [final ordinary-build provenance](../artifacts/plain-binaries-v04-final.json) independently records the same binary hashes and final source identities. Every `emulator_coverage` entry includes exact phases, attempt and HTTP-status counts, pagination observations, result-receipt paths, and any typed probe receipt. The explicit source snapshot includes refreshed hashes and actual method/send line locations.

### Per-operation evidence

Execution labels distinguish production collector requests from direct SDK probes. A `collector_success` status establishes at least one successful result for the operation; the HTTP distribution still includes modeled absence or provider-specific error responses. Cloud Control provider-type coverage is separate from operation-level success.

| SDK operation | Execution | Outcome | Attempts | HTTP status counts | Cursor requests |
| --- | --- | --- | ---: | --- | ---: |
| `cloudcontrol.get_resource` | collector | `collector_success` | 200 | 200 x 200 | 0 |
| `cloudcontrol.list_resources` | collector | `collector_success` | 200 | 200 x 197; 400 x 3 | 0 |
| `cloudfront.list_distributions` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `cloudfront.list_tags_for_resource` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `config.describe_configuration_aggregators` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `config.select_aggregate_resource_config` | collector | `collector_success` | 11 | 200 x 11 | 10 |
| `dynamodb.describe_table` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `dynamodb.list_tables` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ec2.describe_instances` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ec2.describe_internet_gateways` | collector | `collector_success` | 1 | 200 x 1 | 0 |
| `ec2.describe_nat_gateways` | collector | `collector_success` | 1 | 200 x 1 | 0 |
| `ec2.describe_network_acls` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ec2.describe_network_interfaces` | collector | `collector_success` | 3 | 200 x 3 | 2 |
| `ec2.describe_regions` | direct probe | `explicit_sdk_probe_success` | 1 | 200 x 1 | 0 |
| `ec2.describe_route_tables` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ec2.describe_security_groups` | collector | `collector_success` | 3 | 200 x 3 | 2 |
| `ec2.describe_subnets` | collector | `collector_success` | 1 | 200 x 1 | 0 |
| `ec2.describe_volumes` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ec2.describe_vpc_endpoints` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ec2.describe_vpc_peering_connections` | collector | `collector_success` | 1 | 200 x 1 | 0 |
| `ec2.describe_vpcs` | collector | `collector_success` | 1 | 200 x 1 | 0 |
| `ecs.describe_clusters` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `ecs.describe_services` | collector | `collector_success` | 4 | 200 x 4 | 0 |
| `ecs.describe_task_definition` | collector | `collector_success` | 4 | 200 x 4 | 0 |
| `ecs.describe_tasks` | collector | `collector_success` | 4 | 200 x 4 | 0 |
| `ecs.list_clusters` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `ecs.list_services` | collector | `collector_success` | 4 | 200 x 4 | 2 |
| `ecs.list_task_definitions` | collector | `collector_success` | 4 | 200 x 4 | 3 |
| `ecs.list_tasks` | collector | `collector_success` | 6 | 200 x 6 | 2 |
| `eks.describe_cluster` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `eks.describe_nodegroup` | collector | `collector_success` | 4 | 200 x 4 | 0 |
| `eks.list_clusters` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `eks.list_nodegroups` | collector | `collector_success` | 4 | 200 x 4 | 2 |
| `elasticloadbalancingv2.describe_listeners` | collector | `collector_success` | 4 | 200 x 4 | 2 |
| `elasticloadbalancingv2.describe_load_balancers` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `elasticloadbalancingv2.describe_rules` | collector | `collector_success` | 12 | 200 x 12 | 8 |
| `elasticloadbalancingv2.describe_target_groups` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `elasticloadbalancingv2.describe_target_health` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `iam.get_instance_profile` | collector | `collector_success` | 4 | 200 x 4 | 0 |
| `iam.get_policy` | collector | `collector_success` | 6 | 200 x 6 | 0 |
| `iam.get_role` | collector | `collector_success` | 16 | 200 x 16 | 0 |
| `iam.list_attached_role_policies` | collector | `collector_success` | 22 | 200 x 22 | 6 |
| `iam.list_instance_profiles` | collector | `collector_success` | 4 | 200 x 4 | 2 |
| `iam.list_policies` | collector | `collector_success` | 6 | 200 x 6 | 4 |
| `iam.list_roles` | collector | `collector_success` | 16 | 200 x 16 | 14 |
| `lambda.list_functions` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `organizations.list_accounts` | collector | `emulator_not_implemented` | 1 | 501 x 1 | 0 |
| `organizations.list_children` | collector | `emulator_not_implemented` | 3 | 501 x 3 | 0 |
| `organizations.list_organizational_units_for_parent` | collector | `collector_success` | 4 | 200 x 4 | 1 |
| `organizations.list_roots` | collector | `collector_success` | 1 | 200 x 1 | 0 |
| `resourceexplorer2.get_default_view` | direct probe | `explicit_sdk_probe_success` | 1 | 200 x 1 | 0 |
| `resourceexplorer2.get_index` | collector + probe | `sdk_decode_failure` | 3 | 200 x 3 | 0 |
| `resourceexplorer2.get_view` | direct probe | `sdk_decode_failure` | 1 | 200 x 1 | 0 |
| `resourceexplorer2.list_resources` | direct probe | `sdk_decode_failure` | 1 | 200 x 1 | 0 |
| `resourceexplorer2.list_views` | direct probe | `explicit_sdk_probe_success` | 1 | 200 x 1 | 0 |
| `route53.get_hosted_zone` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `route53.list_hosted_zones` | collector | `collector_success` | 2 | 200 x 2 | 1 |
| `route53.list_resource_record_sets` | collector | `collector_success` | 18 | 200 x 18 | 16 |
| `route53.list_tags_for_resource` | collector | `collector_success` | 2 | 200 x 2 | 0 |
| `s3.get_bucket_encryption` | collector | `collector_success` | 5 | 200 x 5 | 0 |
| `s3.get_bucket_logging` | collector | `collector_success` | 5 | 200 x 5 | 0 |
| `s3.get_bucket_notification_configuration` | collector | `collector_success` | 5 | 200 x 5 | 0 |
| `s3.get_bucket_ownership_controls` | collector | `collector_success` | 5 | 200 x 5 | 0 |
| `s3.get_bucket_policy_status` | collector | `collector_success` | 5 | 200 x 2; 404 x 3 | 0 |
| `s3.get_bucket_replication` | collector | `collector_success` | 5 | 200 x 1; 404 x 4 | 0 |
| `s3.get_bucket_tagging` | collector | `collector_success` | 5 | 200 x 1; 404 x 4 | 0 |
| `s3.get_bucket_versioning` | collector | `collector_success` | 5 | 200 x 5 | 0 |
| `s3.get_bucket_website` | collector | `collector_success` | 5 | 200 x 1; 404 x 4 | 0 |
| `s3.get_public_access_block` | collector | `collector_success` | 5 | 200 x 5 | 0 |
| `s3.head_bucket` | direct probe | `explicit_sdk_probe_success` | 1 | 200 x 1 | 0 |
| `s3.list_buckets` | collector | `collector_success` | 5 | 200 x 5 | 4 |
| `sts.get_caller_identity` | collector + probe | `collector_success` | 6 | 200 x 6 | 0 |

### Credential-provider evidence

| SDK operation | Execution | Outcome | Attempts | HTTP status counts |
| --- | --- | --- | ---: | --- |
| `sts.assume_role` | SDK credential provider; outside `ApiExecutor` | `collector_success` | 1 | 200 x 1 |

The provider request occurs only in the `rust:assumed-role` phase. The [assumed-role result](../artifacts/emulator-v04/assumed-role.json) is complete with 38 accounted collector attempts; its additional provider request is intentionally absent from those `ApiExecutor` metrics. The recorder stores no returned credentials, Authorization headers, request bodies, or response bodies.

# Native first-page probe failure triage

## Account prerequisites and remaining wire failures (2026-11-24)

The probe reports `empty` with the provider code when a root scope is known
absent. Sweeps settle those scopes as complete, consistent with other absent
singletons. Generic permission and request errors keep their original classes.
The following rules are operation-scoped:

| Service | Evidence required for an empty scope |
| --- | --- |
| Account | `GetGovCloudAccountInformation` with no `StandardAccountId` queries the caller's linked GovCloud singleton. `ResourceNotFoundException` requires `No GovCloud account is linked to account 000000000000`, with the request's account in place of `000000000000`. Generic account-not-found remains failed. |
| S3 Control | `AccessGrantsInstanceNotExistsError` for the account/Region instance or its grant and location lists. AWS documents the instance as the mandatory container for both lists. |
| Billing Conductor | `AccessDeniedException` with the message for the named operation: `Account 000000000000 is not authorized to call ListBillingGroups. Only payer account is authorized`; `Account 000000000000 is not authorized to call ListCustomLineItems. Only payer account is authorized`; `Account 000000000000 is not authorized to call ListPricingPlans. Only payer account is authorized`; `Account 000000000000 is not authorized to call ListPricingRules. Only payer account is authorized`. Other IAM denies remain inaccessible. |
| Firewall Manager | `AccessDeniedException` with `No default admin could be found for account 000000000000 in Region us-east-1`. AWS documents creating a default administrator as a prerequisite. |
| Macie | `AccessDeniedException` with `Macie is not enabled.` The model also uses this code for insufficient access. |
| Security Hub | `InvalidAccessException` with `Account 000000000000 is not subscribed to Security Hub CSPM in Region us-east-1`. The model also describes this code as an account permission error. |
| Snow Device Management | `ListDevices` with `No devices found in this region.` `ListTasks` remains inaccessible: this message does not establish absence of historical tasks. |
| Security Lake | `ResourceNotFoundException` with `The request failed because Security Lake isn't enabled for your account in any Regions. Enable Security Lake for your account and then try again.` Other missing-resource responses remain failures. |
| ElastiCache | `InvalidParameterValue` with `Use of cache security groups is not permitted in this API version for your account.` Other parameter errors still fail. |
| Redshift | `InvalidParameterValue` with `Amazon Redshift has discontinued cluster security groups. For security group configuration, associate VPC security groups when creating or modifying your cluster.` Other parameter errors still fail. |
| EC2 | `DescribeTrunkInterfaceAssociations` remains failed: `OperationNotPermitted` says the user is not permitted, not that trunk interface associations are absent. |

The exact messages from signed requests to a private emulator 1.0.348 in
`us-east-1` are recorded by service and operation in
[`captured_messages.rs`](../../crates/sweep/src/call/absence/tests/recorded/captured_messages.rs)
and, for GovCloud, in
[`recorded.rs`](../../crates/sweep/src/call/absence/tests/recorded.rs). Tests
contrast the captures with same-operation denials and unrelated not-found
messages. JSON/XML fixture envelopes omit
request IDs and redundant fields; the Firewall Manager fixture preserves the
captured RPC v2 CBOR bytes. All five Firewall Manager messages and all six
Security Hub messages end at `us-east-1`, with no trailing period. For the
Firewall Manager list captures the request included CBOR `MaxResults: 100`.
Both code-only and message rules require HTTP 4xx other than 429. HTTP 429
is throttled and HTTP 503 is transient even if the provider code matches an
absence rule. Message rules require exact text, including the request's account
for Account, Billing Conductor, Firewall Manager, and Security Hub. Firewall
Manager and Security Hub must also name the requested Region. An unreadable
message, conflicting values in the selected error header, or conflicting
header and body codes cannot establish absence.
Code-only rules still accept a recognized header when the body is unreadable,
empty, or does not carry an error code.

[`code_only.rs`](../../crates/sweep/src/call/absence/tests/recorded/code_only.rs)
records signed emulator responses for all three S3 Access Grants reads and
pre-existing absence rules. The emulator returned `NotImplemented` for the
four Organizations checks, so those rules still lack recorded positive responses.

### Resource Explorer `GetIndex`

On a private emulator (1.0.348), a signed `POST /GetIndex` after `CreateIndex`
returned HTTP 200 JSON with `"CreatedAt":1795574719.628248084` and
`"LastUpdatedAt":1795574719.628248084`. The pinned model explicitly sets
`smithy.api#timestampFormat: date-time` on both `GetIndexOutput` members, so both
must be strings. The decoder remains strict; this is a provider wire failure.

### Timestream `DescribeEndpoints`

The pinned AWS JSON 1.0 models share signing name `timestream` and target
`Timestream_20181101.DescribeEndpoints`. Endpoint rules distinguish
`query.timestream.us-east-1.amazonaws.com` from
`ingest.timestream.us-east-1.amazonaws.com`; the emulator override replaces that
authority with loopback. Signed `DescribeEndpoints` requests returned HTTP 500
`AmbiguousService` against the private emulator with loopback and with both
modeled authorities (TLS verification disabled only for the modeled authorities).
Changing only the `X-Amz-Target` operation to `ListScheduledQueries` or
`ListDatabases` returned HTTP 200, even with the two authorities crossed.
Changing the signing name instead to `timestream-query` or `timestream-write`
returned HTTP 404 `UnknownService`. The router can distinguish unique operations
under the shared signing name, but the modeled hostname does not resolve this
ambiguous discovery operation. The collector's modeled request is unchanged.

The first-page probe previously called the data operation directly, so an empty
or item-bearing result did not demonstrate that the native sweep could run the
scope. The probe now looks up the same discovery operation and validates the
same endpoint lease as the sweep. Both fail `ListScheduledQueries` or
`ListDatabases` if `DescribeEndpoints` fails first. `AmbiguousService` has only
been observed from the emulator and now fails immediately; other unknown 5xx
codes still use bounded transient retries.

## Fail-lane checkpoint (2026-11-23)

Pinned models: `46135f8468cd5cb60d983b97290c9f6661e359c6`; native emulator:
1.0.348, extracted at `/tmp/d11-emulator/opt/aws-emulator/bin/aws-emulator`.
No real AWS endpoint was contacted. This checkpoint includes the four fail-lane
service corrections, not other lanes' later catalog changes.

| Checkpoint | Callable rows | Items | Empty | Unsupported | Failed | Skipped child |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Original report / initial reproduced run | 2,134 | 72 | 814 | 485 | 184 | 579 |
| Every-purpose probe, before tag-parent/service corrections | 2,197 | 74 | 832 | 527 | 185 | 579 |
| Fail-lane corrected native run | 2,197 | 74 | 832 | 524 | 125 | 642 |

The 63-row discrepancy is completely explained by the old harness filtering on
`PlanPurpose::Enumeration`: 59 tag endpoints and four bootstrap operations were
omitted. The bootstrap keys are `sts GetCallerIdentity`, `ec2 DescribeRegions`,
`timestream-query DescribeEndpoints`, and `timestream-write DescribeEndpoints`.
It does not require hypothesizing catalog drift. Expand-class inventory operations
also have Enumeration purpose and were already represented.

The native assertion compares the exact set of every callable service/operation
with report keys and rejects duplicate keys/rows. All seven protocol families
returned items. Tag endpoints are genuinely parent-bound even when their special
resource binding is not in ordinary plan bindings; all 59 must be skipped here.
Of those 59, the intermediate run produced 56 failures and three unsupported
responses, explaining the reduction from 527 to 524 unsupported rows.

Original-row transitions: 18 failed→empty, 39 failed→unsupported, four failed→
skipped child, 123 still failed. The extra rows are two successful bootstrap
operations, two failed endpoint discoveries, and 59 skipped tag children.

## Corrections, without protocol fallbacks or concealed failures

- Execute STS identity once and reuse its validated account ID for root bindings.
  Do not substitute an invented account or repeat STS after its evidence row.
- Honor explicit JSON error content types and AWS error headers even for CBOR
  requests. Preserve service error codes; explicit unknown-operation/service
  responses become unsupported. Do not retry using another protocol.
- Recognize narrow absent-feature/singleton codes and retain `empty REASON` in
  probe output. Runtime settlement completes these empty scopes durably; denied
  access and unrelated missing-resource codes remain failures.
- IAM `GetLoginProfile` alone treats `NoSuchEntity` as an absent console login
  profile. Other IAM operations, permission errors and service failures do not.
  Regression evidence includes actual 404 XML decoding and SQLite settlement.
- Batch `ListServiceJobs` binds `jobQueue` from `DescribeJobQueues.jobQueueArn`
  and uses the existing all-status `JOB_NAME=*` filter pattern.
- CloudFormation `ListTypeVersions.Arn` binds from `ListTypes.TypeArn`;
  `ListTypes.Visibility=PRIVATE` explicitly limits discovery to account/region
  extensions (including public extensions activated in that account).
- Kinesis Video `DescribeMappedResourceConfiguration.StreamARN` binds from
  `ListStreams.StreamARN`.
- Redshift `DescribeReservedNodeExchangeStatus.ReservedNodeId` binds from
  `DescribeReservedNodes.ReservedNodeId`. The unrelated deferred
  `GetReservedNodeExchangeOfferings` operation is not promoted.

These four operations stay callable as child expansions, not removed rows. A
first-page root probe does not execute them without their real parents.

## Every remaining failure at the fail-lane checkpoint (125)

### RPC/CBOR serialization failures (77)

All rows below retain `SerializationException`. A signed CodeBuild diagnostic
returned JSON `missing required header x-amz-target` for a valid modeled RPC/CBOR
request. These rows are consistent with an emulator protocol-routing gap, not
proof that their legacy JSON implementations are absent. They remain failures:
`SerializationException` alone is not an unsupported-operation signal, and the
collector does not send an unmodeled JSON fallback. The pinned models advertise
RPC/CBOR and the emitted request follows that selected protocol.

| Service | Operations |
| --- | --- |
| acm-pca | ListCertificateAuthorities |
| cloudhsm-v2 | DescribeBackups |
| codebuild | ListBuildBatches, ListBuilds, ListFleets, ListProjects, ListReportGroups, ListReports, ListSandboxes, ListSourceCredentials |
| codedeploy | ListApplications, ListDeploymentConfigs, ListDeployments, ListOnPremisesInstances |
| codepipeline | ListPipelines, ListWebhooks |
| comprehend | ListDatasets, ListDocumentClassificationJobs, ListDocumentClassifiers, ListDominantLanguageDetectionJobs, ListEndpoints, ListEntitiesDetectionJobs, ListEntityRecognizers, ListEventsDetectionJobs, ListFlywheels, ListKeyPhrasesDetectionJobs, ListPiiEntitiesDetectionJobs, ListSentimentDetectionJobs, ListTargetedSentimentDetectionJobs, ListTopicsDetectionJobs |
| datasync | ListAgents, ListLocations, ListTasks |
| direct-connect | DescribeConnections, DescribeDirectConnectGateways, DescribeInterconnects, DescribeLags, ListResiliencyGroups |
| directory-service | DescribeEventTopics, DescribeSnapshots, DescribeTrusts, ListADAssessments, ListLogSubscriptions |
| memorydb | DescribeACLs, DescribeClusters, DescribeMultiRegionClusters, DescribeParameterGroups, DescribeSnapshots, DescribeSubnetGroups, DescribeUsers |
| network-firewall | ListContainerAssociations, ListFirewallPolicies, ListFirewalls, ListRuleGroups, ListTLSInspectionConfigurations, ListVpcEndpointAssociations |
| route-53-domains | ListDomains |
| service-catalog | ListPortfolios, ListProvisionedProductPlans, ListServiceActions, ListTagOptions |
| servicediscovery | ListOperations, ListServices |
| shield | ListAttacks, ListProtectionGroups, ListProtections |
| textract | ListAdapterVersions, ListAdapters |
| workspaces | DescribeApplications, DescribeConnectionAliases, DescribeIpGroups, DescribeWorkspaceBundles, DescribeWorkspaceDirectories, DescribeWorkspaceImages, DescribeWorkspaces, DescribeWorkspacesPools, ListAccountLinks |

### Permission/account eligibility failures (20)

Do not convert these to empty inventories merely because this emulator account
lacks a service prerequisite. Nineteen rows decode to access-denied classes;
EC2 returns the distinct fatal `OperationNotPermitted` code.

| Service | Operations | Evidence |
| --- | --- | --- |
| billingconductor | ListBillingGroups, ListCustomLineItems, ListPricingPlans, ListPricingRules | AccessDeniedException |
| fms | GetNotificationChannel, ListAppsLists, ListPolicies, ListProtocolsLists, ListResourceSets | AccessDeniedException |
| macie2 | ListCustomDataIdentifiers, ListMembers | AccessDeniedException |
| securityhub | DescribeActionTargets, DescribeHub, GetEnabledStandards, ListAutomationRules, ListFindingAggregators, ListMembers | InvalidAccessException |
| snow-device-management | ListDevices, ListTasks | AccessDeniedException |
| ec2 | DescribeTrunkInterfaceAssociations | OperationNotPermitted; signed request at modeled maxResults=255 says this user is not permitted |

### ImageBuilder validation mismatch (11)

`ListComponentBuildVersions`, `ListComponents`, `ListContainerRecipes`,
`ListDistributionConfigurations`, `ListImagePipelines`, `ListImageRecipes`,
`ListImages`, `ListInfrastructureConfigurations`, `ListLifecyclePolicies`,
`ListWorkflowBuildVersions`, `ListWorkflows` fail with
`InvalidParameterValueException`. The pinned model permits maxResults=100;
the emulator requires at most 25. A diagnostic at 25 then returned NotImplemented.
Do not change valid model-derived request sizes to mask this emulator mismatch.

### Other individual failures (17)

| Service / operation | Cause / bounded diagnostic |
| --- | --- |
| account / GetGovCloudAccountInformation | HTTP 404 ResourceNotFoundException: `No GovCloud account is linked to account 000000000000`. The account must match the request; generic not-found remains failed. |
| bcm-pricing-calculator / ListBillScenarios | ValidationException at valid maxResults=25: operation forbidden for member accounts. |
| bedrock-agentcore-control / ListBrowsers | Malformed emulator response: numeric createdAt/updatedAt where the pinned shape explicitly requires date-time. Strict timestamp regression rejects it; no coercion. |
| chime-sdk-voice / ListPhoneNumbers | BadRequestException: emulator accepts maxResults≤99; pinned ResultMax permits 100. |
| controltower / ListEnabledBaselines | ValidationException: missing/unassumable AWSControlTowerAdmin role. |
| controltower / ListEnabledControls | ResourceNotFoundException: landing zone must be created first. |
| elasticache / DescribeCacheSecurityGroups | InvalidParameterValue: `Use of cache security groups is not permitted in this API version for your account.` |
| marketplace-agreement / SearchAgreements | ValidationException; diagnostic at valid maxResults=50 rejects the filter combination. No account-scoping filter is invented. |
| mediapackage / ListOriginEndpoints | UnprocessableEntityException: emulator caps maxResults at 50; pinned model permits 1,000. |
| redshift / DescribeClusterSecurityGroups | InvalidParameterValue: `Amazon Redshift has discontinued cluster security groups. For security group configuration, associate VPC security groups when creating or modifying your cluster.` |
| s3-control / GetAccessGrantsInstance | AccessGrantsInstanceNotExistsError: no account instance. |
| s3-control / ListAccessGrants | AccessGrantsInstanceNotExistsError: no account instance. |
| s3-control / ListAccessGrantsLocations | AccessGrantsInstanceNotExistsError: no account instance. |
| securitylake / ListSubscribers | ResourceNotFoundException: `The request failed because Security Lake isn't enabled for your account in any Regions. Enable Security Lake for your account and then try again.` Verified with modeled `GET /v1/subscribers`. |
| ssm-contacts / ListRotations | ValidationException: account not found for the request. |
| timestream-query / DescribeEndpoints | AmbiguousService: loopback emulator cannot disambiguate the endpoint-discovery request. No alternate target/protocol is invented. |
| timestream-write / DescribeEndpoints | AmbiguousService, same routing ambiguity as query. |

## Reproduction and actual checks

All commands ran from `/tmp/w4-fail`, with `CARGO_TARGET_DIR=/tmp/tgt-fail` and
`TMPDIR=/tmp/w4-fail/.local/fail/tmp`. Generated binary/reports were refreshed
only in that worktree and are not part of the implementation commits.

- `cargo fmt -p catalog-tool -p aws-wire -p inventory-sweep -p inventory-harness`: passed.
- `cargo clippy --locked -j3 -p catalog-tool -p aws-wire -p inventory-sweep -p inventory-harness --all-targets -- -D warnings`: passed.
- `cargo test --locked -j3 -p catalog-tool --lib given_service_jobs_when_planned_then_all_queues_and_statuses_are_covered -- --nocapture`: one passed.
- `cargo test --locked -j3 -p catalog-tool --test cloudformation_required_parent --test kinesis_video_required_parent --test redshift_required_parent -- --nocapture`: four passed. An initial Redshift test named the unrelated deferred GetReservedNodeExchangeOfferings and failed; its target was corrected to the actual fixed operation, not weakened.
- `cargo run --locked -j3 -p catalog-tool -- models compile --write`: passed.
- `INVENTORY_EMULATOR_BINARY=/tmp/d11-emulator/opt/aws-emulator/bin/aws-emulator cargo test --locked -j3 -p inventory-harness --features native-emulator-tests --test probe_native -- --nocapture`: one passed, 133.41 seconds; exact 2,197-key coverage and seven item-bearing protocol families.
- `cargo test --locked -j3 -p aws-wire --lib response::errors -- --nocapture`: 18 passed.
- `cargo test --locked -j3 -p aws-wire --lib response::timestamp -- --nocapture`: four passed.
- `cargo test --locked -j3 -p inventory-harness --test probe -- --nocapture`: six passed.
- `cargo test --locked -j3 -p inventory-sweep --lib call::absence::tests -- --nocapture`: four passed, including actual IAM XML 404 versus AccessDenied/ServiceFailure.

Logs and captured response headers/bodies are retained under `.local/fail/`.
No workspace-wide suite, real AWS test, complete child-expansion inventory, or
SQLite/Postgres/FalkorDB end-to-end acceptance run is claimed by this probe.
The storage regression exercises durable SQLite scope settlement separately.

After the IAM-specific addition, `cargo fmt -p inventory-sweep` and
`cargo clippy --locked -j3 -p inventory-sweep --all-targets -- -D warnings` passed.
`cargo test --locked -j3 -p inventory-sweep --lib execution::page::settlement::tests -- --nocapture`
passed all three scenarios, including a missing IAM login profile completing its
durable scope and an IAM permission denial leaving that scope inaccessible.

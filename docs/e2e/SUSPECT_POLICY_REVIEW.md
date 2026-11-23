# Suspect-callable model review (2026-11-23)

## Evidence and interpretation

Reviewed the advisory `suspect_callable=219` total from `/tmp/svc-verdicts.txt` against the pinned Smithy models under `.local/aws-models/models/<service>/service/` and the effective embedded policy, including late tag/endpoint overrides. Advisory files are not authorization. The model revision is the repository-pinned api-models-aws revision, not a current online SDK.

The advisory counts are not a reliable one-to-one claim index: EMR names two callable defects but reports one; CodeArtifact names a second low-severity binding omission but reports one; SSM discusses a second unscoped call but reports one. Some sections include denied or deferred operations in their mistake lists. The table below names the actual allegations rather than inventing operations to make their counts fit. `C` means CONFIRMED and corrected here; `R` means REFUTED as an established overlay defect. `R` does **not** prove every possible live response is complete. Model optionality alone, hypothetical sharing, duplicate read cost, and speculative endpoint behavior do not establish a defect. Expected-absence runtime findings are explicitly separated from policy findings.

Given the pinned model and an effective policy, when a confirmed policy defect is corrected, `crates/catalog-tool/tests/suspect_policy.tsv` records the resulting class, literal, binding, identity, tag path, scope, or branch contract. The Rust test loads effective overlays, not intermediate tables. These are policy contracts, not a claim of emulator or live-account execution.

## Per-service dispositions

The number in parentheses is the advisory verdict count, retained verbatim for reconciliation.

| Service (count) | Claimed operations and disposition | Pinned-model / effective-policy evidence |
|---|---|---|
| account (1) | R policy: GetGovCloudAccountInformation; runtime expected absence confirmed | Documentation says linked GovCloud account “if it exists”; modeled ResourceNotFoundException is not a reason to disable a valid read. Runtime classification belongs to the failure lane. |
| amp (1) | C DescribeScraper | Nested source configuration includes `configurationBlob` (blob). Queryable paths are not output projection. Denied Content rather than pretending opaque configuration is safe. |
| apigatewayv2 (1) | R ListPortalProducts | `ResourceOwner` is an unconstrained string; model supplies neither SELF as a valid literal nor proof unfiltered output includes AWS reference products. Do not invent SELF. |
| app-mesh (5) | C ListGatewayRoutes, ListRoutes, ListVirtualGateways, ListVirtualNodes, ListVirtualServices | Parent MeshRef supplies `meshOwner`; all five requests accept it. Bind the owner's account together with the mesh name. |
| appconfig (2) | C ListExtensions; R ListDeploymentStrategies | ListExtensions explicitly includes AWS-authored extensions and has no owner selector: defer it and dependent GetExtension. The deployment-strategy allegation is not established by the pinned list contract. |
| appstream (5) | R DescribeApplications, DescribeStacks, DescribeImageBuilders, DescribeImages, DescribeUsageReportSubscriptions | Optional ARN/key members do not prove missing response identities. DescribeImages already pins PRIVATE. Changing keys or dropping whole resource families without observed missing values is not justified. |
| appsync (2) | C ListGraphqlApis; R ListChannelNamespaces | `owner=CURRENT_ACCOUNT` is modeled. ListChannelNamespaces operation documentation explicitly excludes code handlers despite sharing a shape containing them. |
| arc-region-switch (2) | R ListPlans, ListPlansInRegion | Endpoint model is regional. A control-plane trait or regionless ARN alone does not prove either list is globally scoped. |
| batch (2) | C ListServiceJobs; R ListQuotaShares | ListServiceJobs defaults to RUNNING without filters; bind each owned queue and send modeled JOB_NAME `*`, matching ListJobs. QuotaShareDetail optionality alone is not observed absence. |
| bedrock (1) | C ListCustomModels | Modeled owner filter distinguishes account custom models from shared models; pin owned models. |
| bedrock-agentcore-control (7) | C ListBrowsers, ListCodeInterpreters, GetBrowser, GetCodeInterpreter, ListEvaluators; R ListMemories, ListAgentRuntimes | Browser/interpreter type CUSTOM excludes SYSTEM and fixes dependent reads. Evaluators explicitly mix BUILTIN/THIRD_PARTY with no modeled request filter: defer. Memory optional ARN and alleged runtime-version ARN reuse are not demonstrated failures. |
| bedrock-data-automation (2) | R ListBlueprints, ListDataAutomationProjects | Stage/optional-member observations do not establish collisions or omitted identities in these list results. |
| chatbot (4) | R DescribeChimeWebhookConfigurations, DescribeSlackChannelConfigurations, ListCustomActions, ListMicrosoftTeamsChannelConfigurations | Service documentation lists regional APIs and storage in each relevant Region; Ohio processing does not establish a global resource collection. |
| cleanroomsml (1) | R ListAudienceGenerationJobs | Optional scoping/accessible-resource language does not establish foreign ownership of returned account job records. |
| clouddirectory (1) | R ListDirectories | DELETED is an example lifecycle value, not proof of the default. Pinning ENABLED would silently exclude disabled directories. |
| cloudformation (1) | C ListTypeVersions | Request requires Arn or Type+TypeName conditionally even though members are individually optional. Bind Arn from ListTypes.TypeArn. ListTypes visibility defaults PRIVATE. |
| cloudsearch (1) | R DescribeDomains | DomainStatus ARN optionality is not proof of omission. Do not replace it with an unverified invented ARN format. |
| cloudtrail (1) | C ListDashboards | Modeled CUSTOM filter excludes AWS-provided dashboards. |
| cloudwatch (3) | R DescribeAnomalyDetectors, ListAlarmMuteRules, ListMetricDataQueryDefinitions | First two allegations are optional identity members; third is absence of documented ARN format, not evidence that the existing format is wrong. DescribeInsightRules and denied GetMetricDataQueryDefinition observations are not additional counted callable defects. |
| cloudwatchomni (2) | R ListDomains, ListIntegrations | Optional ARN alongside required ID is not evidence the list omits ARN. |
| codeartifact (1) | C DescribeRepository; defensive DescribeDomain binding | RepositorySummary.domainOwner must accompany domainName/name. Domain list is already caller-owned; explicitly carry its owner as well. |
| codedeploy (3) | R ListDeploymentConfigs, GetDeploymentConfig, ListDeployments | Pinned docs do not establish AWS-owned configurations in the list. Historical growth is not a policy defect under ALL resources. |
| controltower (1) | C ListEnabledControls | Modeled includeChildren defaults false; set true to cover child OUs. Contrary to the advisory, this member has documentation. |
| cost-and-usage-report-service (1) | R DescribeReportDefinitions | Model/endpoint report is regional; us-east-1-only assertion was not established by the pinned contract. |
| database-migration-service (2) | R DescribeFleetAdvisorCollectors, DescribeFleetAdvisorDatabases | Model EOS notice does not demonstrate a failed read or authorize dropping supported modeled operations. Data-provider settings do not reach endpoint password shapes. |
| dataexchange (1) | C ListDataSets | Origin OWNED is modeled and distinguishes owned from entitled data sets. |
| datazone (1) | R ListEnvironments | Optional summary id alone is not proof of omitted runtime identity. |
| deadline (2) | R ListBudgets, ListLicenseEndpoints | Sensitive free-form budget description is not a credential; licenseEndpointId optionality is not observed absence. Queryable is not a projection. |
| detective (1) | R ListOrganizationAdminAccounts | No model evidence establishes multiple graph memberships for the same administrator in this list. A membership-key analogy alone is insufficient to change identity. |
| direct-connect (3) | R DescribeDirectConnectGatewayAssociationProposals, DescribeDirectConnectGatewayAssociations, DescribeDirectConnectGatewayAttachments | Existing expansions correctly cover owned DXGWs. Additional VGW/TGW/VIF routes are coverage work, not invalid existing policy. |
| docdb-elastic (1) | R ListClusterSnapshots | Deleted-cluster snapshot retention and default snapshotType truncation are explicitly unverified by the advisory and not established in the pinned model. |
| ecs (1) | R ListAccountSettings | Account configuration is within account inventory. Optional principalArn does not demonstrate absent output on effective settings. |
| efs (2) | R DescribeFileSystems, DescribeAccessPoints | Optional ARN members do not establish identity absence. |
| elasticsearch-service (1) | R DescribePackages | “Available” does not establish AWS-public package ownership. No modeled owner literal can be invented safely. |
| emr (1) | C ListNotebookExecutions, ListStudioSessionMappings | From defaults to 30 days ago: set epoch start. StudioId request documentation scopes mappings to a Studio: bind ListStudios.StudioId. The advisory counts only one despite naming two. |
| eventbridge (1) | R ListEndpoints | Account-wide/home-region suspicion lacks pinned-model proof of a scope mismatch. |
| eventbridgev2 (1) | C DescribeSubscriber | HttpParameters can contain forwarded authentication/header/query values; cannot project that whole nested value with the existing one-text digest. Deny Secret. |
| gamelift (2) | C ListContainerGroupDefinitions, DescribeMatchmakingConfigurations | Nested environment overrides can carry credentials; matchmaking GameSessionData carries customer content. Deny Secret and Content respectively. |
| global-accelerator (1) | R DescribeAccelerator | A redundant detail read is not incorrect inventory and does not justify dropping a valid callable. |
| guardduty (1) | R ListMalwareProtectionPlans | Optional ID and explicit NextToken without paginated trait are not defects: the members exist and no missing ID response is shown. |
| iam (2) | R policy GetLoginProfile, ListRoleTags; runtime expected absence confirmed | NoSuchEntity is an expected absence to classify at runtime. Redundant tags remain a correct parent-joined observation. |
| identitystore (3) | R DescribeIdentityStore, ListGroups, ListIdentityStores | Home-region/default account assumptions need live scope evidence; no pinned-model proof of wrong routing was established. |
| interconnect (1) | R ListConnections | Accessible does not establish that foreign-owned connections are returned as owned resources. |
| iot (1) | R ListJobTemplates | AWS-managed templates have a distinct ListManagedJobTemplates API. General list wording is not proof the custom list includes them. |
| iot-managed-integrations (3) | R ListAccountAssociations, ListManagedThings, ListCredentialLockers | Optional ARN does not demonstrate absence. Names, brands, serial numbers, owner labels and model-sensitive display text are not password/token values. No speculative digest/deletion of inventory metadata. |
| iotfleetwise (5) | R ListSignalCatalogs, ListModelManifests, ListDecoderManifests, ListCampaigns, ListStateTemplates | All five counted allegations are optional ARN observations, not demonstrated omissions. Separately hardened all six modeled listResponseScope-bearing lists with METADATA_ONLY; the advisory incorrectly said no operations use the enum. |
| kinesis-video (1) | C DescribeMappedResourceConfiguration | StreamARN must select the parent stream. Identity now correlates request StreamARN with selected Type+ARN, preserving associations across streams. |
| kms (4) | R ListKeys, ListAliases, DescribeCustomKeyStores, ListGrants | Tiny page size is inefficient, not incorrect. KeyId+GrantId is a valid contextual key. DescribeKey remains authoritatively denied: docs say it can create AWS-managed keys. |
| lakeformation (1) | R ListLFTags | LFTag does include CatalogId in the pinned model. The missing-member allegation is false. |
| lambda (2) | R ListLayers, ListLayerVersions | Pinned ListLayers has no owner selector, but neither its documentation nor the advisory establishes that public/AWS-owned layers are included. Dependent ListLayerVersions inherits the same unproven allegation. ListFunctions remains denied for Environment.Variables. |
| license-manager-user-subscriptions (3) | R ListIdentityProviders, ListInstances, ListLicenseServerEndpoints | Optional identity members are contract risks, not demonstrated missing runtime values; SecretId is a reference. |
| managedblockchain (1) | C ListNetworks | IsOwned=true is modeled and excludes networks merely accessible through other accounts. |
| marketplace-agreement (1) | R SearchAgreements | Queryable is output metadata, not a request field or output projection. No credentials are demonstrated by the queryable-field allegation. |
| migrationhubstrategy (1) | R ListAnalyzableServers | hostname/ipAddress/source/vmId are individually optional; model does not establish the alleged duplicate-hostname response or a guaranteed alternative composite identifier. |
| neptune (2) | R DescribeDBClusterSnapshots, DescribeGlobalClusters | IncludePublic/IncludeShared snapshot defaults are explicitly false. GlobalCluster ARN optionality is not observed absence. |
| network-firewall (1) | R ListRuleGroups | Scope default is explicitly ACCOUNT, not MANAGED. |
| networkmanager (4) | C ListCoreNetworks; R ListAttachments, ListPeerings, ListConnectPeers | Core network list explicitly says “owned and shared” without owner filter: defer mixed root. OwnerAccountId fields alone do not establish foreign output for the three other lists. |
| networkmonitor (1) | R GetMonitor | Probe ARN optionality is not observed omission; existing fixtures assume ARN. |
| notifications (1) | R ListManagedNotificationConfigurations | Account-bearing configuration ARNs and list documentation do not establish a global AWS reference catalog returned identically to all accounts. |
| notificationscontacts (2) | C ListEmailContacts, GetEmailContact | Global endpoint is notifications-contacts.us-east-1.api.aws; modeled contact ARN has no Region. Set Account scope on both. Email address PII is not a secret credential. |
| observabilityadmin (2) | R ListTelemetryRulesForOrganization, ListCentralizationRulesForOrganization | These are admin-only organization control-plane policies, like Organizations metadata. SourceAccountIds/destination accounts describe policy coverage, not proof of foreign ownership of policy resources. Organization scope remains explicit. |
| opensearch (1) | C ListDomainNames | EngineType is optional filtering; removing OpenSearch-only filter retains Elasticsearch-engine domains too. |
| organizations (12) | R policy all twelve listed below; runtime expected absence confirmed | Organization/account/root/admin/service/status/transfer metadata belongs to the caller's organization. AWSOrganizationsNotInUseException, ResourcePolicyNotFoundException, AccountNotRegisteredException require runtime expected-absence handling, not twelve policy demotions. |
| partnercentral-revenue-measurement (1) | R ListRevenueAttributions | Optional/unpatterned ARN does not demonstrate missing or malformed actual ARN values. AWS vs Sandbox is a separate explicit catalog-scope decision. |
| partnercentral-selling (2) | R ListEngagements, ListSolutions | Optional ARN does not establish missing identity. Catalog AWS is an existing scope choice; not proof of malformed callable policy. |
| pinpoint-sms-voice-v2 (5) | C DescribePhoneNumbers, DescribePools, DescribeSenderIds, DescribeOptOutLists, DescribeRcsAgents | Modeled Owner=SELF excludes shared resources. |
| qconnect (2) | R ListTagsForResource, ListAIPromptVersions | Tags are joined via late TagEndpoint policy; redundant reads remain correct. Optional outer version summary is not observed absence. Deferred presigned/content operations were not promoted. |
| quicksight (1) | R ListTopics/ListTopicsV2 duplicate-read allegation | Same resource ARN across two read operations is not proof the persistent resource identity is duplicated. Explicit NextToken members support paging without a model paginator trait. |
| ram (2) | R ListPermissionAssociations, ListSourceAssociations | Pinned permission-owner documentation defaults SELF, contrary to the advisory; explicitly pin SELF defensively. Source associations are share metadata, not evidence of returning foreign physical resources. |
| redshift (7) | R DescribeReservedNodes, DescribeSnapshotCopyGrants, DescribeScheduledActions, DescribeQev2IdcApplications, DescribeRedshiftIdcApplications, DescribeInboundIntegrations, DescribeDataShares | Pinned output shapes and effective overlays already carry the correct Tags path. Contract rows preserve all seven existing tag paths. |
| resiliencehubv2 (2) | C ListServices, ListPolicies | Modeled accountId filters service account/policy owner. Bind caller Account on both. |
| resource-explorer-2 (1) | R policy GetIndex/ListStreamingAccessForServices; runtime expected absence confirmed | ResourceNotFound index is an empty-account runtime case; service-principal grants are account configuration. No invalid class is established by the advisory. |
| route-53 (7) | R ListCidrCollections, ListReusableDelegationSets, GetReusableDelegationSet, GetDNSSEC, ListHostedZonesByName, ListTrafficPolicyInstancesByHostedZone, ListTrafficPolicyInstancesByPolicy | Four optional-identity allegations are not observed missing members; three redundant scans are correct reads. Cost is not corruption. |
| route-53-domains (1) | R ListTagsForDomain | Effective late tag plan already sets FixedRegion us-east-1; the alleged missing routing override is false. |
| route53-recovery-control-config (1) | R ListTagsForResource | Endpoint rules already route all request Regions to the partition's control endpoint. Parent-bound tags are not independent regional enumerations; no incorrect data is demonstrated. |
| route53profiles (1) | R ListTagsForResource | Late TagEndpoint plan supplies parent binding and identity merge. Intermediate deferred text is not effective policy. |
| route53resolver (1) | R ListFirewallDomainLists | Operation docs say lists “that you have defined”; ManagedOwnerName in a shared shape alone does not prove AWS reference lists are returned. |
| s3-control (1) | R policy GetAccessGrantsInstance; runtime expected absence confirmed | The singleton can legitimately be absent. AccessGrantsInstanceNotFound requires a narrow runtime empty/absence rule rather than disabling valid inventory. Intermediate deferred text for ListMultiRegionAccessPoints is overridden correctly. |
| s3tables (1) | R ListTableBuckets | Documentation says buckets “for your account.” AWS-managed account buckets are physical resources, not AWS's public reference catalog. |
| sagemaker (4) | C ListAliases, ListCandidatesForAutoMLJob, DescribeTrainingPlanExtensionHistory; R ListTags | Add request ImageName/AutoMLJobName/TrainingPlanArn to child-local identity. Late tag plan already joins owning resources. |
| sagemaker-geospatial (1) | R ListEarthObservationJobs | EarthObservationJobSummary does include Tags; preserve the existing effective tag path. |
| schemas (3) | C ListRegistries, inherited ListSchemas; R ListTagsForResource | Scope=Local excludes AWS registries. Effective late TagEndpoint is valid despite intermediate deferred decision/report omissions. |
| security-ir (2) | R ListMemberships, ListTagsForResource | Optional membershipArn is not observed omission; late tags correctly bind the same ARN. |
| securityagent (1) | C ListSecurityRequirementPacks | Modeled filter managementType=CUSTOMER_MANAGED excludes managed reference packs. |
| securityhub (5) | C ListOrganizationAdminAccounts; R ListConnectors, ListConnectorsV2, ListInvitations, ListTagsForResource | Feature defaults to CSPM; explicitly enumerate both modeled SecurityHub and SecurityHubV2 branches. Other allegations are optional identity members or a late tag-plan misunderstanding. |
| securitylake (1) | R ListSubscribers | STS ExternalId is a confused-deputy discriminator, not a credential. Its presence does not establish secret disclosure; SecretArn-like references likewise remain metadata. |
| service-catalog (1) | C ListProvisionedProductPlans | Modeled AccessLevelFilter Key=Account, Value=Self avoids default per-user visibility. |
| service-quotas (1) | C ListRequestedServiceQuotaChangeHistory | RequestedLevel=ALL avoids default ACCOUNT-only history. |
| servicediscovery (2) | C ListServices; R ListOperations | RESOURCE_OWNER filter SELF excludes namespaces shared by other accounts. Async operation summaries are permitted account metadata, not unsafe reads. |
| snow-device-management (3) | R ListDevices, ListTasks, ListTagsForResource | Optional identities/parent ARNs are not observed absence. Tags remain parent-joined, even if redundant with inline tags. |
| snowball (1) | C ListJobs | Model explicitly says one US endpoint returns jobs in all US Regions. No current scope represents that geography without regional misattribution: defer honestly. |
| socialmessaging (1) | R ListTagsForResource | Effective late TagEndpoint is parent-bound and valid. Deferred mutation comments are not callable-policy defects. |
| ssm (1) | C DescribePatchBaselines; R additional ListOpsItemRelatedItems claim | OWNER=Self is a modeled patch filter. Optional OpsItemId permits the unscoped account list; no model proof of invalid inventory is supplied. |
| ssm-contacts (3) | C ListRotationShifts, ListPageReceipts, ListPageResolutions | Computed shifts keyed only by two-value Type collide: defer. Receipt identity needs ReceiptTime to distinguish repeated acknowledgements, but timestamp identity representation is unsupported: explicitly defer rather than omit that dimension. Resolution key includes PageId and required ContactArn+Type, not nullable StageIndex. |
| ssm-incidents (1) | R ListTagsForResource | Late tag plan already overrides intermediate deferred text with a valid parent-joined expansion. |
| ssm-sap (2) | R ListDatabases, ListTagsForResource | Optional application/component filters do not establish invalid global listing. Inline-plus-endpoint tags are redundant but correct. |
| storage-gateway (1) | R ListAutomaticTapeCreationPolicies | GatewayARN is optional in the request; the advisory supplies no default/completeness contract proving input-less enumeration invalid. |
| support-app (2) | R ListSlackChannelConfigurations, ListSlackWorkspaceConfigurations | Advisory explicitly requires a two-region live comparison; no pinned proof establishes account-global rows behind regional endpoints. |
| swf (1) | R ListDomains | Optional DomainInfo.arn is not observed absent identity. |
| synthetics (1) | R ListTagsForResource | Cross-region behavior was explicitly unverified. Existing parent-joined tags are not proven to be sent to an invalid endpoint. |
| verifiedpermissions (6) | C ListPolicyStores, ListPolicyStoreAliases, ListIdentitySources, ListPolicies, ListPolicyTemplates, GetPolicyStore | Five maxResults docs cap at 50 even though numeric trait lacks max: set page_size=50. GetPolicyStore returns tags only when input tags=true. |
| wafv2 (5) | C GetWebACL, GetIPSet, GetRegexPatternSet, GetRuleGroup, GetManagedRuleSet | Ordinary From bindings ignored parent branch provenance. Each child branch now uses FromBranch matching REGIONAL/CLOUDFRONT. Same additional defect fixed for GetLoggingConfiguration. |
| wellarchitected (3) | C ListProfiles; R ListWorkloads, ListReviewTemplates | ProfileOwnerType=SELF is modeled. Owner fields/share invitation fields alone do not establish foreign workloads/templates returned by those unfiltered calls. |
| workspaces (3) | C DescribeWorkspaceImages; R DescribeWorkspaceDirectories, GetAccountLink | ImageType=OWNED excludes shared images. RegistrationCode identifies directory registration and is not established as a bearer credential. Optional/redundant link output does not prove a bad read. |

Organizations' twelve counted operations: DescribeOrganization, DescribeResourcePolicy, ListAccounts, ListRoots, ListCreateAccountStatus, ListDelegatedAdministrators, ListAWSServiceAccessForOrganization, ListInboundResponsibilityTransfers, ListOutboundResponsibilityTransfers, DescribeAccount, DescribeCreateAccountStatus, ListDelegatedServicesForAccount.

## All-callable risk scan

`scripts/audit-callable-risk.rb` traverses the pinned shapes reachable from every callable operation in the baseline current overlay report, not just the advisory services. Run from the repository root:

```sh
ruby scripts/audit-callable-risk.rb
```

Observed baseline: **2,197 callable operations across 340 services**; **98 owner/scope-like input members in 91 operations**; **62 secret-ish output member paths in 39 operations**. These are candidates, not automatic denials. Traversal visits each reachable shape once per operation, so the output contains representative paths, not all paths through reused shape types. Pagination tokens, secret ARNs, redacted fields, labels, and credentials are distinguished by model documentation. For example FSx FsxAdminPassword says always redacted; EC2 registered SQL credentials are Secrets Manager ARNs. Do not deny either by member name alone. Generated `catalog/callable-risk.tsv` is intentionally untracked.

## Execution boundary and remaining evidence

Ran the Ruby model/risk traversal and model-document extraction. After coordinator authorization, ran the following commands in `/tmp/w4-suspect`; all exited 0. The final full test run after catalog regeneration reported **908 passed, 10 ignored across 70 suites** (36.67 seconds). Ignored tests were not executed.

```sh
cargo fmt
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo clippy -j3 -p catalog-tool --all-targets --locked -- -D warnings
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo test -j3 -p catalog-tool --locked
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo run -j3 -p catalog-tool --locked -- models compile --write
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo run -j3 -p catalog-tool --locked -- models compile --check
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo run -j3 -p catalog-tool --locked -- overlays generate --check
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo run -j3 -p catalog-tool --locked -- kinds generate --check
```

Earlier verification found missing Input imports, obsolete engine/count snapshots, and unsupported receipt timestamp identity; the fix-up commits correct those findings. Two initial Clippy attempts were blocked by OS122 disk quota before local compilation (one used `RUSTC_WRAPPER= CARGO_INCREMENTAL=0`); coordinator cleanup resolved that quota. The pre-regeneration suite reached integration tests and failed because the ignored catalog still advertised the now-denied AMP DescribeScraper. Regenerating the ignored catalog fixed that publication mismatch.

Regenerate the full merged probe following RUN_HERE.md. No emulator probes or AWS calls were run in this lane. No coverage success is claimed for deferred mixed-owner roots, receipt timestamp identity, computed shifts, Snowball geographic scope, or opaque/secret-bearing outputs. Existing expected-absence handling (Organizations, account GovCloud linkage, IAM login profiles, Resource Explorer index) remains runtime-lane work. Optional response identities, disputed cross-region behavior, additional Direct Connect association routes, and sandbox catalogs remain explicitly unproven; this review does not substitute an empty emulator response for item-bearing account evidence.

## Patch manifest

Policy/test commits, in application order (the documentation-manifest commit follows these):

```text
05fc6ab0 cf0b0066 cacd801f 2e26e0fe f3e69652 cab8270c 73cb2a7d fe33f0df
44ae3e7d 88e69e90 08aa0f75 02f2abb1 cb0619b8 352a6619 dfa3579e 2fc73694
723b214f 701ce9b9 a9bd16d9 02f6f66a 66f519b7 952c7a45 2784182c 34b68814
7767bd1f dd5ba2ef 3bd911b6 f2c7a37b c0b2fe6a 103c57cd 806a4c5f 6cc535f4
7f67c5e2 1a24a10c 5343e91d 49f5772f 6714f0ec 9805459b 1038c9d6 c0e73e5b
ea3f8df6 74e5843b 944c3c01 e8df3ef2 dacdec1a 377e8d8f 280cf617 b60b54d8
```

Verification fix-up commits: `445e08ef` (owner filter Input imports), `9ced4214` (canonical formatting), `4b26df58` (SSM Input import and bounded assertion helpers), `9a632b48` (engine/secret-denial snapshots), `b172b5c4` (receipt timestamp deferral). Documentation verification evidence follows these commits.

Exact changed paths, relative to repository root:

```text
scripts/audit-callable-risk.rb
docs/e2e/SUSPECT_POLICY_REVIEW.md
crates/catalog-tool/tests/suspect_policy.rs
crates/catalog-tool/tests/suspect_policy.tsv
crates/catalog-tool/src/overlay/builtin/family_a_c/bedrock_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/bedrock_agentcore_control_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/appsync_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/cloudtrail_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/cloudformation_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/codeartifact_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/amp_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/app_mesh_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/appconfig_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/batch_0.rs
crates/catalog-tool/src/overlay/builtin/family_a_c/controltower_0.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/s_eventbridgev2.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/tests.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/s_dataexchange.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/support.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/s_emr.rs
crates/catalog-tool/src/overlay/builtin/family_g_k/kinesis_video.rs
crates/catalog-tool/src/overlay/builtin/family_g_k/s_kinesis_video.rs
crates/catalog-tool/src/overlay/builtin/family_g_k/iotfleetwise.rs
crates/catalog-tool/src/overlay/builtin/family_g_k/gamelift.rs
crates/catalog-tool/src/overlay/builtin/family_l_p/managedblockchain.rs
crates/catalog-tool/src/overlay/builtin/family_l_p/pinpoint_sms_voice_v2.rs
crates/catalog-tool/src/overlay/builtin/family_l_p/networkmanager.rs
crates/catalog-tool/src/overlay/builtin/family_l_p/opensearch.rs
crates/catalog-tool/src/overlay/builtin/family_l_p/notificationscontacts.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/construction.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/schemas/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/ram/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/service_quotas/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/sagemaker/calls_7.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/securityagent/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/service_catalog/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/ssm_contacts/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/ssm_contacts/decisions_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/ssm/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/servicediscovery/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/resiliencehubv2/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/securityhub.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/snowball/calls_0.rs
crates/catalog-tool/src/overlay/builtin/family_q_s/snowball/decisions_0.rs
crates/catalog-tool/src/overlay/builtin/family_t_z/wellarchitected_plans_1.rs
crates/catalog-tool/src/overlay/builtin/family_t_z/workspaces_plans_1.rs
crates/catalog-tool/src/overlay/builtin/family_t_z/verifiedpermissions_plans_1.rs
crates/catalog-tool/src/overlay/builtin/family_t_z/endpoint_plans.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/tests_d_f_misc.rs
crates/catalog-tool/src/overlay/builtin/family_d_f/tests_d_f_registry.rs
crates/catalog-tool/src/overlay/builtin/family_l_p/consumer_tests.rs
```

Compatibility: owner filters deliberately remove shared/reference observations; Content/Secret denials and mixed-owner/computed/geographic/timestamp-identity deferrals reduce callable coverage honestly. Correlated identities change keys for mapped-stream associations, SageMaker child records, SSM contact resolutions, and Security Hub feature administrators. Notifications contact scope changes regional keys to account keys. Existing persisted snapshots may therefore require reconciliation under the new catalog; this patch does not migrate stored data.

## WAFv2 sweep-path branch regression

`policy_tz_completion_tests/wafv2.rs` compiles the real pinned WAFv2 policies and drains durable SQLite work through the existing sweep request/page/child harness. Its native-request fake transport serves both REGIONAL and CLOUDFRONT branches for WebACLs, IP sets, regex pattern sets, rule groups, managed rule sets, and logging configurations. It checks exactly one list and one detail request for each resource family in each branch (24 requests total), with no cross-branch sibling requests.

Home Region is deliberately eu-west-1. Every named detail request must retain the parent Scope, Name and Id; CloudFront requests must use wafv2.us-east-1.amazonaws.com while regional requests use wafv2.eu-west-1.amazonaws.com. Logging details have no modeled Scope input: their ResourceArn must instead retain the correct parent branch. The test also checks the two durable WebACL ResourceKeys, one regional and one account-global.

Positive and mutation command:

```sh
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo test -j3 -p inventory-sweep --locked given_wafv2_scope_branches_when_sweeping_then_only_matching_child_requests_persist
```

The positive run passed (1 test, 462 filtered out). Mutated `endpoint_plans::detail_branches` once from `Binding::FromBranch` to the prior unqualified `Binding::From`; the same test failed (exit 101), observing wafv2.eu-west-1.amazonaws.com where the CloudFront parent required wafv2.us-east-1.amazonaws.com. Restored `FromBranch` immediately afterward. The mutation is not committed. This is a fake-transport sweep regression, not emulator or live-AWS evidence.

Additional changed paths for this regression:

```text
crates/sweep/src/execution/page/child/policy_tz_completion_tests.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests/catalog.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests/wafv2.rs
crates/sweep/src/execution/page/child/policy_tz_completion_tests/wafv2/transport.rs
```

Runtime regression commit: `25620c57`. Crate gate commands (the package name is `inventory-sweep`):

```sh
cargo fmt
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo clippy -j3 -p inventory-sweep --all-targets --locked -- -D warnings
CARGO_TARGET_DIR=/tmp/tgt-suspect cargo test -j3 -p inventory-sweep --locked
```

The first gate invocation passed formatting and all-target Clippy. Its full sweep test run exceeded the 600-second command deadline; before the deadline, the log reported stale fixture failures for WAF branch provenance, App Mesh parent pairs, and mixed search engines. This timed-out invocation is not a passing full-suite result.

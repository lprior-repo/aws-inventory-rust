# Seeded sweep: all 112 failed scopes

This attributes the failures in [seed-oracle-failed-scopes.tsv](seed-oracle-failed-scopes.tsv), not a successful all-service run. The complete row-by-row result is [failed-scope-triage.tsv](failed-scope-triage.tsv). `source_line` is the original TSV line number, so the four otherwise identical Resolver rows remain distinguishable. Every row has a root-cause key resolved below and captured request evidence.

| Classification | Scopes | Root causes |
| --- | ---: | --- |
| our defect | 1 | Marketplace's documented-required search filter was absent |
| emulator gap | 100 | 77 CBOR dispatch; 13 pagination bounds; 3 timestamp encodings; 4 managed Resolver tag lookups; 3 Timestream endpoint-discovery failures |
| missing seed prerequisite | 6 | GovCloud link; Security Lake enablement; Incident Manager account setup; 3 S3 Access Grants instance reads |
| needs policy change | 5 | account-instance SSO; member-account billing API; 2 retired security-group APIs; EC2 operation permission |
| **Total** | **112** | No failures were converted to successful empty inventories or silently skipped |

## Capture and reproduction

- Model pin: `aws-models` commit `46135f8468cd5cb60d983b97290c9f6661e359c6` from `models.lock`.
- Emulator: pinned `1.0.348`, own process on port `46163`; own TLS capture proxy on `38797`; own graph server on `17443`. No other lane's emulator was stopped or modified.
- Independent client: AWS CLI `2.40.3`. Its JSON protocol selection differs from the inventory catalog's supported CBOR selection; this is important evidence, not an instruction to change the production protocol.
- Native seed replay: 41 of 52 selected recipes executed, exit 0. A fresh sweep restricted to the 36 services represented by the original failures returned exit 2: 371 scopes, 224 complete, 35 unsupported, **112 failed**, 274 resource writes, zero remaining work, zero pending graph writes, 406 attempts and 12 retries. This replay captured the failures; it was not a post-fix acceptance run.
- The native harness probe exited 0 as a diagnostic command; that does not mean all probed operations passed.
- Local raw evidence is `/tmp/w4-triage/.local/triage/wire/requests.jsonl`; `capture_id` identifies its numbered transactions. The TSV retains exact request-body bytes for each non-Resolver row. `(empty)` means zero bytes, not `{}`. The protocol/path rules below plus those bytes specify the failing requests without authentication material.
- Raw SDK debug logs and credential files are deliberately not committed. Only protocol headers, emulator account/resource identifiers and request payloads are recorded here.

## CBOR-routing: 77 scopes, emulator gap

All 77 requests received HTTP 400 with the same body:

```json
{"__type":"SerializationException","message":"missing required header `x-amz-target`"}
```

Every one used this captured envelope (operation and service shape supplied by the tables):

```http
POST /service/<service-shape>/operation/<operation>
content-type: application/cbor
accept: application/cbor
smithy-protocol: rpc-v2-cbor
```

There is **no** `x-amz-target`. This matches the pinned service's `smithy.protocols#rpcv2Cbor` trait and the [Smithy RPC v2 CBOR specification](https://smithy.io/2.0/additional-specs/protocols/smithy-rpc-v2-cbor.html), which prohibits that header. `bfff` is a valid indefinite-length empty CBOR map, not a missing body. Nonempty examples include `bf6a4d6178526573756c74731864ff` (`{"MaxResults":100}`) and the full nested filters recorded in the TSV. Even empty-map requests fail before shape deserialization; this is not a timestamp, blob, empty-list, enum, member-casing or CBOR collection-encoding defect.

All 17 pinned models also support AWS JSON: JSON 1.1 except Network Firewall, which supports JSON 1.0. The emulator's protocol selection chooses AWS JSON first and requires its target header, rather than dispatching the advertised CBOR path/header. Independent CLI requests use `POST /`, `content-type: application/x-amz-json-1.1` (Network Firewall: `1.0`), the target shown below and body `{}`. They reach application dispatch. A JSON success does not prove CBOR is invalid.

| Service | CBOR service shape | Native representative operation / capture | CLI target / capture / HTTP |
| --- | --- | --- | --- |
| shield | AWSShield_20160616 | ListProtections / 934 | AWSShield_20160616.ListProtections / 1318 / 400 subscription absent |
| direct-connect | OvertureService | DescribeDirectConnectGateways / 829 | OvertureService.DescribeDirectConnectGateways / 1319 / 200 |
| route-53-domains | Route53Domains_v20140515 | ListDomains / 876 | Route53Domains_v20140515.ListDomains / 1320 / 200 |
| acm-pca | ACMPrivateCA | ListCertificateAuthorities / 1118 | ACMPrivateCA.ListCertificateAuthorities / 1321 / 200 |
| datasync | FmrsService | ListTasks / 1119 | FmrsService.ListTasks / 1322 / 200 |
| memorydb | AmazonMemoryDB | DescribeACLs / 1120 | AmazonMemoryDB.DescribeACLs / 1323 / 200 |
| textract | Textract | ListAdapters / 1150 | Textract.ListAdapters / 1324 / 200 |
| codebuild | CodeBuild_20161006 | ListProjects / 1158 | CodeBuild_20161006.ListProjects / 1304 / 200 |
| codedeploy | CodeDeploy_20141006 | ListApplications / 1171 | CodeDeploy_20141006.ListApplications / 1335 / 200 |
| comprehend | Comprehend_20171127 | ListEntityRecognizers / 1178 | Comprehend_20171127.ListEntityRecognizers / 1325 / 501 NotImplemented |
| workspaces | WorkspacesService | DescribeWorkspaces / 1205 | WorkspacesService.DescribeWorkspaces / 1326 / 200 |
| cloudhsm-v2 | BaldrApiService | DescribeBackups / 1200 | BaldrApiService.DescribeBackups / 1327 / 501 NotImplemented |
| codepipeline | CodePipeline_20150709 | ListPipelines / 1219 | CodePipeline_20150709.ListPipelines / 1328 / 200 |
| service-catalog | AWS242ServiceCatalogService | ListPortfolios / 1264 | AWS242ServiceCatalogService.ListPortfolios / 1329 / 200 |
| network-firewall | NetworkFirewall_20201112 | ListFirewalls / 1265 | NetworkFirewall_20201112.ListFirewalls / 1330 / 200 |
| servicediscovery | Route53AutoNaming_v20170314 | ListServices / 1279 | Route53AutoNaming_v20170314.ListServices / 1331 / 200 |
| directory-service | DirectoryService_20150416 | DescribeTrusts / 1282 | DirectoryService_20150416.DescribeDirectories / 1332 / 200 |

The Directory Service comparison is a second operation on the same service, not a claim that `DescribeTrusts` was called by the CLI. All 77 operation-specific native payloads are retained in the TSV. Fix emulator protocol dispatch; do not add a target header to CBOR or silently downgrade production requests. The independent JSON calls also expose later failures (Shield subscription, Comprehend and CloudHSM implementation gaps); repairing dispatch alone is not evidence that all 77 scopes will complete.

## Pagination bounds: 13 scopes, emulator gaps

The pinned Smithy input ranges are inclusive. The sweep's modeled maximum and wire spelling are correct. Independent CLI calls using the same limits reproduce each service's rejection (captures 1309, 1312, 1317; each CLI exits 254).

| Root-cause key | Scopes | Exact native request | Pinned limit | Observed rejection |
| --- | ---: | --- | --- | --- |
| ImageBuilder-page-limit | 11 | `POST /<operation>`, `content-type: application/json`, `{"maxResults":100}`; ListImages, ListWorkflows, ListComponents, ListImageRecipes and ListContainerRecipes additionally send `"owner":"Self"` | maxResults 1..100 | HTTP 400 InvalidParameterValueException: `maxResults must be smaller than or equal to 25` |
| Chime-page-limit | 1 | `GET /phone-numbers?max-results=100`, empty body, capture 1252 | maxResults 1..100 | HTTP 400 BadRequestException: `Member must have value less than or equal to 99` |
| MediaPackage-page-limit | 1 | `GET /origin_endpoints?maxResults=1000`, empty body, capture 1232 | MaxResults 1..1000 | HTTP 422 UnprocessableEntityException: `maxResults must be an integer between 1 and 50` |

Image Builder captures 1217, 1220, 1222, 1231, 1236, 1240..1245 correspond to the 11 operations listed individually in the TSV. No request-side clamp was added: that would hide an emulator/model disagreement rather than fix inventory behavior.

## date-time-response: 3 scopes, emulator gap

These are successful HTTP responses with incorrectly typed timestamp members. The model explicitly requires `smithy.api#timestampFormat: date-time`; this conclusion does not rely on a guessed REST JSON default. The inventory decoder correctly rejects numeric epochs instead of silently accepting malformed responses.

| Scope / capture | Exact request (empty body) | Explicit format trait | HTTP 200 response excerpt |
| --- | --- | --- | --- |
| connectcases GetDomain / 1230 | `POST /domains/f5685f35-7ccd-46e9-b1a2-8616314779ec` | GetDomainResponse.createdTime targets CreatedTime, whose shape carries date-time | `"createdTime":1795461428.421343981` |
| internetmonitor GetMonitor / 1262 | `GET /v20210603/Monitors/inventory-thin-internetmonitor` | GetMonitorOutput.CreatedAt and ModifiedAt members carry date-time | `"CreatedAt":1795461441.389047384,"ModifiedAt":1795461441.389047384` |
| resource-explorer-2 GetIndex / 1308 | `POST /GetIndex` | GetIndexOutput.CreatedAt and LastUpdatedAt members carry date-time | `"CreatedAt":1795461452.562430302,"LastUpdatedAt":1795461533.572041735` |

## managed-firewall-tags: 4 scopes, emulator gap

`ListFirewallDomainLists` (capture 1271) publishes four built-in managed lists and one seeded list. The native driver passes each **returned ARN unchanged** to `ListTagsForResource`. The four managed lists fail; the seeded list succeeds with `{"Tags":[]}` (capture 1292).

All failing requests are `POST /`, `content-type: application/x-amz-json-1.1`, `x-amz-target: Route53Resolver.ListTagsForResource`, with body `{"MaxResults":100,"ResourceArn":"arn:aws:route53resolver:us-east-1:000000000000:firewall-domain-list/<ID>"}`:

| Capture | ID from the preceding list response | HTTP 400 response |
| --- | --- | --- |
| 1283 | rslvr-fdl-1bc724e346019cfb | ResourceNotFoundException; ResourceType FirewallDomainList |
| 1288 | rslvr-fdl-4ab5894cbe1429d4 | ResourceNotFoundException; ResourceType FirewallDomainList |
| 1296 | rslvr-fdl-cb4f4cfd74cafd8a | ResourceNotFoundException; ResourceType FirewallDomainList |
| 1299 | rslvr-fdl-d57b208eeaea0f26 | ResourceNotFoundException; ResourceType FirewallDomainList |

The emulator's managed-list catalog is separate from its persisted-resource tag lookup; the tag operation returns not-found without consulting that catalog. Independent CLI `get-firewall-domain-list --firewall-domain-list-id rslvr-fdl-4ab5894cbe1429d4` succeeds (capture 1334), while `list-tags-for-resource --resource-arn <same returned ARN>` fails (capture 1336, HTTP 400, CLI exit 254). This rules out an invented/nonexistent ID or native ARN extraction error. It does not establish what real AWS's managed-resource tagging policy should be; the observed emulator inconsistency is the attribution.

The original TSV omits scope input identities, so its four duplicate rows cannot be matched individually to the four fresh IDs. All four rows reference the complete reproduced set rather than inventing a one-to-one mapping.

## Timestream-endpoint-routing: 3 scopes, emulator gap

The root scopes `timestream-query/ListScheduledQueries`, `timestream-write/ListDatabases`, and `timestream-write/ListBatchLoadTasks` fail during required endpoint discovery, **before those list requests are sent**. The shared bootstrap request is:

```http
POST /
content-type: application/x-amz-json-1.0
x-amz-target: Timestream_20181101.DescribeEndpoints

{}
```

It receives HTTP 500:

```json
{"__type":"AmbiguousService","message":"2 emulated services all claim requests for 'timestream': timestream, timestream; the request is rejected rather than served by an arbitrary one"}
```

The pinned discovery operation and shared `timestream` signing identity explain the request. Captures 1280, 1281, 1286, 1287, 1290, 1291, 1294, 1295, 1298, 1300, 1301, 1302, 1303, 1305 and 1306 are discovery attempts/retries shared across the two services, not three independently attributed list requests. An additional direct diagnostic found `ListBatchLoadTasks` unimplemented, but that is not the recorded root cause of these three failures.

## Missing seed prerequisites: 6 scopes

| Root-cause key / scopes | Exact request and capture | Response evidence / required prerequisite |
| --- | --- | --- |
| GovCloud-link / 1 | `POST /getGovCloudAccountInformation`, application/json, `{}` / 847 | HTTP 404 ResourceNotFoundException: `No GovCloud account is linked to account 000000000000`; requires a linked GovCloud account |
| Access-Grants-instance / 3 | `GET /v20180820/accessgrantsinstance/grants?maxResults=1000` / 1186; `GET /v20180820/accessgrantsinstance` / 1194; `GET /v20180820/accessgrantsinstance/locations?maxResults=1000` / 1202; all empty body, `x-amz-account-id: 000000000000` | HTTP 404 AccessGrantsInstanceNotExistsError: `Access Grants Instance does not exist`; requires CreateAccessGrantsInstance before reads |
| Security-Lake-enabled / 1 | `GET /v1/subscribers?maxResults=100`, empty body / 1237 | HTTP 404 ResourceNotFoundException: Security Lake is not enabled in any Region; enable a data lake first |
| Incident-Manager-account / 1 | `POST /`, JSON 1.1, target SSMContacts.ListRotations, `{"MaxResults":1024}` / 1238 | HTTP 400 ValidationException: `Invalid value provided - Account not found for the request`, Reason OTHER; emulator requires Incident Manager account/replication-set setup |

These classify the observed read failure, not a claim that all prerequisite creation APIs are supported. The sweep recognizes the GovCloud result as empty only for an HTTP client error other than 429 with the captured `ResourceNotFoundException` message naming the request's account. An unrelated not-found message or a missing-link message for another account remains failed. The seed-repair lane separately recorded that Security Lake enablement encounters organization/account restrictions and unimplemented delegated-administrator/leave-organization operations. Those setup gaps remain failures, not proof of a successful seed. No blanket ResourceNotFound/ValidationException suppression was added.

## Account, resource-kind and operation policy: 5 scopes

| Root-cause key / scope | Exact request and capture | Response evidence / policy decision required |
| --- | --- | --- |
| organization-instance-only / sso-admin ListPermissionSets | `POST /`, JSON 1.1, target SWBExternalService.ListPermissionSets, `{"InstanceArn":"arn:aws:sso:::instance/ssoins-7223b1d1ac5e1f0e","MaxResults":100}` / 1188 | HTTP 400 ValidationException: `This operation is not supported for account instances of IAM Identity Center.` Distinguish organization instances from account instances; do not globally ignore validation errors. |
| payer-or-regular-account / bcm-pricing-calculator ListBillScenarios | `POST /`, JSON 1.0, target AWSBCMPricingCalculator.ListBillScenarios, `{"maxResults":25}` / 840 | HTTP 400 ValidationException, reason INVALID_REQUEST_FROM_MEMBER; API allows regular/payer accounts, not this member account. |
| trunk-permission / ec2 DescribeTrunkInterfaceAssociations | `POST /`, application/x-www-form-urlencoded, `Action=DescribeTrunkInterfaceAssociations&Version=2016-11-15&MaxResults=255` / 1069 | HTTP 400 OperationNotPermitted: `User 000000000000 is not permitted to perform this operation`. Payload parsing succeeds; permission/account applicability needs explicit policy. |
| retired-security-groups / redshift DescribeClusterSecurityGroups | `POST /`, application/x-www-form-urlencoded, `Action=DescribeClusterSecurityGroups&Version=2012-12-01` / 1160 | HTTP 400 InvalidParameterValue: `Amazon Redshift has discontinued cluster security groups. For security group configuration, associate VPC security groups when creating or modifying your cluster.` |
| retired-security-groups / elasticache DescribeCacheSecurityGroups | `POST /`, application/x-www-form-urlencoded, `Action=DescribeCacheSecurityGroups&Version=2015-02-02` / 1227 | HTTP 400 InvalidParameterValue: `Use of cache security groups is not permitted in this API version for your account.` |

These rows are not authorization-policy fixes in this patch. Changing operation selection or treating an unsupported account/resource kind as terminal requires a reviewed inventory policy; guessing an identifier or converting these failures to empty success would mask the problem.

## Marketplace-required-filter: confirmed inventory defect and fix

`SearchAgreements` sent `POST /service/AWSMPCommerceService_v20200301/operation/SearchAgreements` using the CBOR envelope above and body `bf6a6d6178526573756c74731832ff` (`{"maxResults":50}`), capture 831. The emulator **does** implement CBOR for this service: the response is a CBOR ValidationException with reason UNSUPPORTED_FILTERS and a `filters` field error, not the target-header routing error.

The pinned input models `filters` as optional structurally, but its documentation requires the AgreementType filter for the supported search combinations. The fix supplies the modeled input through the existing catalog overlay:

```json
{"filters":[{"name":"AgreementType","values":["PurchaseAgreement"]}]}
```

The filter is not emulator-specific and does not change the wire protocol or suppress errors. Independent AWS CLI `marketplace-agreement search-agreements --filters name=AgreementType,values=PurchaseAgreement` sends JSON 1.0 to target AWSMPCommerceService_v20200301.SearchAgreements and receives HTTP 200 `{"agreementViewSummaries":[]}` (capture 1333, CLI exit 0).

The same documented-required/model-optional pattern was searched across all **109 distinct failed operations**. Other matches concern conditional input pairs, pagination, ordering or ownership; CodeDeploy applicationName/deploymentGroupName and Workspaces UserName/DirectoryId may legitimately both be absent. Redshift's legacy API documents either ClusterSecurityGroupName or Marker but is explicitly discontinued in this account; no fake root identifier or continuation token was added. Marketplace is the only confirmed unconditional missing-input fix in these failures.

### Executable regression scenarios added (not executed in this lane)

- Given the pinned Marketplace model and overlay, when the runtime catalog is compiled, then its fixed input contains the exact modeled lower-case `filters/name/values` structure.
- Given that actual compiled catalog and a fake external transport, when the driver consumes a page, persists it, reopens the scenario and requests the continuation page, then both independently decoded CBOR requests retain the filter and maxResults 50, the resumed request carries nextToken, and both canonical account-global agreement IDs persist with the scope complete.
- Given an empty search response through fake transport, when the driver drains the scope, then the request still contains the filter and the completed scope stores zero agreements.

These use the repository's real catalog compiler, runtime request builder, execution driver and persistence helpers. The external AWS boundary alone is faked; no emulator-echo oracle is used. Test execution/fmt/clippy are reserved for the coordinator; these scenarios are **unverified**, not claimed green. Generated catalog/database artifacts must be regenerated together for deployment; an old compiled catalog retains the old missing filter until replaced.

## Emulator gap list (seed-gap TSV style)

The entries below extend the seed-repair lane's gap-list style with sweep evidence. The scope TSV is the authoritative membership list for grouped operations.

```tsv
service	operation	gap	evidence
17 services in CBOR table	77 operations in scope TSV	Advertised rpcv2Cbor is routed to AWS JSON target-header validation	All return HTTP400 SerializationException missing x-amz-target; service-specific native/CLI captures above
imagebuilder	11 List operations in scope TSV	Emulator caps maxResults at25 but pinned model permits100	Native1217,1220,1222,1231,1236,1240-1245; SDK1309
chime-sdk-voice	ListPhoneNumbers	Emulator rejects100 though pinned inclusive maximum is100	Native1252; SDK1312
mediapackage	ListOriginEndpoints	Emulator caps maxResults at50 but pinned model permits1000	Native1232; SDK1317
connectcases	GetDomain	createdTime is numeric despite explicit date-time model trait	HTTP200 capture1230
internetmonitor	GetMonitor	CreatedAt/ModifiedAt are numeric despite explicit date-time model traits	HTTP200 capture1262
resource-explorer-2	GetIndex	CreatedAt/LastUpdatedAt are numeric despite explicit date-time model traits	HTTP200 capture1308
route53resolver	ListTagsForResource	Managed domain lists are published but absent from tag resource lookup	List1271; tags1283,1288,1296,1299; independent get1334/tags1336
timestream-query,timestream-write	DescribeEndpoints prerequisite for3 root scopes	Both emulated services claim timestream and discovery is rejected as ambiguous	HTTP500 shared bootstrap captures1280-1306
comprehend	ListEntityRecognizers	Secondary gap after JSON dispatch: operation unimplemented	SDK1325 HTTP501
cloudhsm-v2	DescribeBackups	Secondary gap after JSON dispatch: operation unimplemented	SDK1327 HTTP501
timestream-write	ListBatchLoadTasks	Secondary direct-probe gap; not the3 recorded bootstrap failures	Native diagnostic returned NotImplemented
```

No aws-wire production encoding changes, generic retries, fallback protocols, emulator-specific inputs or ignored-error policies were introduced.

## Commands actually run

The following used existing diagnostic binaries, not a build of this patch. All were run in `/tmp/w4-triage` using the own-emulator credentials; no credentials are reproduced here.

```bash
/tmp/tgt-int/debug/inventory-harness seed run \
  --seeds crates/harness/seeds --catalog /tmp/w4-oracle/catalog/aws-api.catalog \
  --endpoint https://127.0.0.1:38797 --ca-bundle .local/triage/wire/cert.pem \
  --region us-east-1 --concurrency 8 \
  --oracle .local/triage/oracle.tsv --matrix .local/triage/matrix.tsv
# Exit 0.

# Native inventory-harness probe used the same catalog/endpoint/CA/region,
# --report .local/triage/probe.report and 33 explicit --service selections.
# Exit 0; per-operation outcomes still include failures.

args=()
while IFS= read -r service; do args+=(--service "$service"); done \
  < <(cut -f1 docs/e2e/build/seed-oracle-failed-scopes.tsv | tail -n +2 | sort -u)
/home/lewis/.cache/tgt-oracle/debug/aws-inventory sweep \
  --database .local/triage/sweep.sqlite --catalog /tmp/w4-oracle/catalog/aws-api.catalog \
  --profile default --region us-east-1 --endpoint https://127.0.0.1:38797 \
  --ca-bundle .local/triage/wire/cert.pem \
  --graph-url redis://127.0.0.1:17443 --graph-name triage --drain "${args[@]}"
# Exit 2; 112 failed scopes as reported above.
```

AWS CLI comparisons used `--no-cli-pager --no-paginate --endpoint-url https://127.0.0.1:38797 --ca-bundle .local/triage/wire/cert.pem --region us-east-1`; the service operations and observed responses are recorded in the tables above. The initial CLI spelling `codedeploy list-applications` was invalid (exit 252), then corrected to `deploy list-applications` (exit 0). An initial redirected Resolver command had an invalid log filename containing a slash, made no request, and was corrected before captures 1335/1336. Neither failed invocation is treated as API evidence.

No Cargo build, formatter, clippy, test suite or post-patch sweep was run in this implementation lane. Coordinator gates remain required for catalog-tool, inventory-sweep, inventory-harness and aws-wire, including `cargo fmt --check`, clippy with `-D warnings`, the added regression scenarios and regeneration of the catalog/database pair.

# Account and inventory-source discovery

The `inventory_aws::discovery` module supplies a bounded, read-only bootstrap
layer for native collectors. It discovers Organizations accounts and their
hierarchy, describes existing Config aggregators, and inspects an existing
Resource Explorer index and views. Each result carries explicit coverage and
selection evidence. Discovery never creates a role, policy, recorder, aggregator,
index, view, or delegated-administrator relationship.

## Public API

With an AWS-enabled build, the CLI exposes the same report:

```sh
aws-inventory discover --profile organization-audit --region us-east-1 \
  --organization --hierarchy --config-aggregators --resource-explorer
```

`discover` emits JSON and does not start a resource scan. Its default source
selection, when none of the three source flags is supplied, checks Config and
Resource Explorer in the requested region. Organizations enumeration is explicit.

```rust,ignore
use std::sync::Arc;
use inventory_aws::discovery::{DiscoveryOptions, discover};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};

let executor = Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
let report = discover(
    DiscoveryOptions {
        profile: Some("organization-audit".into()),
        region: "us-east-1".into(),
        organization: true,
        hierarchy: true,
        config_aggregators: true,
        resource_explorer: true,
        max_pages: 10_000,
        page_size: 100,
    },
    Arc::clone(&executor),
).await?;

// Keep report.complete and the individual coverage records with these results.
let eligible = report.organization.as_ref()
    .map(|organization| organization.eligible_accounts().count())
    .unwrap_or(0);
```

`discover` returns `Result<DiscoveryReport, AwsScanError>`. Invalid options,
configuration-loading failure, cancellation before bootstrap, and inability to
resolve the caller identity return `Err`. Once a caller identity is known,
service listing failures remain in the returned report with any preceding
observations. A denied `ListAccounts` is therefore an incomplete account set,
never a successful empty organization.

The caller supplies one endpoint region. Organizations calls use the SDK's
Organizations endpoint resolver; Config and Resource Explorer calls inspect
the explicit region only. Regional source discovery must be requested in each
region of interest. It does not infer that an index or aggregator in one region
exists in another region.

`supplemental_sources(&Context, config_enabled, explorer_enabled)` provides the
same source selection to the scan coordinator without allocating an unused
event receiver. It borrows the scan's configuration, identity, region, bounds,
and shared executor. The public report-only command has no event channel.

## Report interpretation

`DiscoveryReport.complete` means that the requested discovery traversals
finished without a recorded failure. It does **not** mean that a resource
inventory is complete, that discovered roles can be assumed, or that a chosen
Config/Explorer source contains every resource.

Every requested listing has `DiscoveryCoverage`:

| Field | Meaning |
| --- | --- |
| `complete` | All requested pages/branches in this traversal reached their normal terminal response without a recorded failure. |
| `pages` | Paginated responses passed to the cursor, including empty pages; responses rejected before cursor processing are excluded. This is not a count of all AWS requests or retry attempts. |
| `items_seen` | Raw items in accepted response pages before mapper validation and duplicate rejection. A page that would exceed the item budget is rejected before its item count is accepted. |
| `cancelled` | At least one observed failure was executor cancellation. |
| `failures` | Bounded operation, parent ID, category, and error descriptions. |
| `suppressed_failures` | Additional diagnostic records omitted after the fixed record limit. Completeness remains false. |

Point reads such as `GetIndex` and `GetView` can add failure evidence without
incrementing `pages`. Use the executor's attempt counters for admitted explicit
API attempts. SDK credential-provider requests form a separate boundary,
described below.

`SourceSelection` is serialized with a `status` tag:

| Status | Meaning and collection behavior |
| --- | --- |
| `not_requested` | This discovery source was disabled. No selection is available. |
| `absent` | A completed check found no existing usable source of this kind. No infrastructure is enabled. |
| `selected` | An identifier and the explicit selection reason are available. Its declared scope still limits what it can report. |
| `ambiguous` | Multiple candidates exist without an authoritative default. Candidate identifiers are retained; none is selected. |
| `unavailable` | Selection could not be established safely, such as denied/incomplete enumeration, a non-ACTIVE index, or an inconsistent default. |

An ambiguous result can have complete discovery coverage: all candidates were
observed, but the program has insufficient selection information. Likewise, an
observed index in `CREATING` state is a completed observation and an unavailable
inventory source. Callers must inspect both coverage and selection.

## Organizations account and hierarchy contract

Account enumeration uses `ListAccounts` and the current `State` field. Only
`State == ACTIVE` sets `eligible_for_scan = true`. Suspended, closed, pending,
unknown, and missing states remain in the report and are ineligible for
automatic scanning. No fallback interprets the retired `Status` field as
current eligibility. AWS documents the `Status` retirement and the required
move to `State` in the [ListAccounts API reference][accounts].

The mapper retains ID, optional ARN and name, state, joined method, and joined
timestamp. Account email addresses are not copied into discovery reports.
Account eligibility does not establish successful cross-account access;
assumption and collector failures remain separate scan outcomes.

When hierarchy is requested, the adapter performs:

1. `ListRoots` to identify root containers.
2. `ListOrganizationalUnitsForParent` for each observed root/OU.
3. `ListChildren` with `ChildType = ACCOUNT` for account membership in that
   same parent.

Traversal is breadth first. Each parent's OU and account listings are joined
as two owned futures; parent traversal itself is sequential and bounded. The
result retains root and OU nodes, OU parent IDs/depths, and account-to-parent
memberships. Duplicate node IDs, multiple parents for one account, excessive
depth, and an unexpectedly empty successful root listing produce partial
coverage. Successful observations from sibling requests survive another
branch's denial or failure.

This is an observation assembled over multiple requests, not a transactional
organization snapshot. Accounts can move while discovery runs. The adapter
does not infer that a missing membership deletes a previously observed
relationship, nor does it add an account found only in a hierarchy branch to
the list of ACTIVE accounts approved for scanning.

Organizations explicitly warns that its list operations can return **empty
pages with a non-null continuation token**. The shared discovery cursor follows
the token until a terminal response; it does not stop on an empty result set.
The Organizations adapters clamp requested page size to the documented maximum
of 20. See [ListAccounts][accounts], [ListOrganizationalUnitsForParent][units],
and [ListChildren][children].

## Existing Config aggregators

`DescribeConfigurationAggregators` enumerates existing aggregators in the
caller's account and the selected region. Each accepted aggregator ARN must
match that partition, account, region, and Config service namespace. The report
retains its name/ARN, declared account source groups, organization-source
presence, declared regions/all-regions flags, and whether aggregator filters
exist. See the [Config API reference][config].

A complete list with no candidates is `absent`; a complete list with exactly
one candidate selects that candidate; multiple candidates are `ambiguous`.
An incomplete list is `unavailable`, even if one candidate was seen before
the failure. The program does not choose the first item in a partial or
multi-aggregator list.

Automatic selection identifies an existing source owned by the caller in the
requested endpoint region. It does not establish that the source records all
requested accounts, regions, or resource types. A centrally managed aggregator
can legitimately include accounts beyond the caller. Its declared scope and
filter-presence metadata must remain visible when interpreting inventory
results. The supplemental collector preserves the account/region reported in
each configuration item. Explicit aggregator selection remains available when
the operator needs a particular source.

## Existing Resource Explorer sources

Discovery first calls `GetIndex`. A typed `ResourceNotFoundException` is an
absent index; an access denial is unavailable with failure evidence. The index
ARN, type, state, and replication regions are retained. The adapter proceeds
to view selection only for an `ACTIVE` index. It never creates an index or
changes its type. See [GetIndex][index].

`ListViews` enumerates same-account, same-region view ARNs. `GetDefaultView`
then establishes the AWS regional default, if any. Selection follows these
rules:

1. A default is selected only if its ARN occurs in the completed view listing.
2. Without a regional default, exactly one observed view may be selected.
3. Multiple views without a default are ambiguous.
4. A default absent from the completed listing is explicitly unavailable.
5. The selected view must pass `GetView` with the expected ARN and matching
   owner, when the owner field is present.

Discovery does not enumerate shared views or assume a cross-account view is
the appropriate default. It rejects source ARNs from another account or region.
This keeps an unrelated view out of automatic selection. The selected view's
scope and included property names remain in the report. Search-filter text is
marked sensitive by the SDK and is omitted; only `has_filter` is retained.
See [GetDefaultView][default-view] and [GetView][view].

A filtered view, a local index, or index replication can limit the resources
available to queries. A selected view is supplementary evidence, not a proof
that every AWS service and region is represented.

## Cross-account credentials

`account_role_arn(partition, account_id, role_name_or_path)` constructs an ARN
for an explicitly supplied existing IAM role. It validates a 12-digit account
ID, partition syntax, and a bounded role name/path. Wildcards, empty/relative
path segments, control characters, and URL escapes are rejected. A leading
slash is rejected; pass `audit/Inventory`, not `/audit/Inventory`.

`assume_role_config(&SdkConfig, &AssumeRoleOptions, &ApiExecutor)` installs the
official `aws_config::sts::AssumeRoleProvider`. It keeps a provider instead of
turning one STS response into static credentials. The AWS identity cache can
therefore refresh credentials for a long-running collection. The caller must
pass a loaded base configuration with credentials, time, and region providers.
No role discovery or role/policy creation occurs.

```rust,ignore
use inventory_aws::discovery::{AssumeRoleOptions, account_role_arn, assume_role_config};

let options = AssumeRoleOptions {
    role_arn: account_role_arn("aws", "123456789012", "audit/Inventory")?,
    session_name: "inventory-run".into(),
    external_id: None,
    duration_seconds: 3_600,
};
let member_config = assume_role_config(&base_config, &options, &executor).await?;
```

Session names and external IDs follow bounded STS-compatible syntax. Session
duration is deliberately limited to 900–3600 seconds so callers using role
chaining stay within its one-hour limit. The general STS API allows longer
sessions for other cases; that larger range is outside this helper's contract.
Cross-account assumption requires both a suitable existing target trust policy
and permission for the source identity to assume that role. The [AssumeRole API
reference][assume-role] describes those requirements.

Credential setup is cancellation-aware and has a 30-second deadline. Provider
construction is lazy: successfully returning a configuration does not prove
that STS will authorize assumption. The consuming request will resolve that
outcome. The credential provider's internal STS configuration has one SDK
attempt, at most 30 seconds for an operation/attempt/read, and at most 10
seconds for connection establishment; stricter base timeouts are retained.
These credential-specific timeouts do not replace the returned collector
configuration's timeout settings.

The provider's internal STS requests, including later refresh, do **not** pass
through the inventory executor. They are not counted as inventory API attempts
and do not share its per-bucket pacing. The explicit inventory call consuming
those credentials remains executor-admitted, cancellation-aware, and bounded
by the executor's timeout. Any work managed internally by the SDK is governed
by the SDK provider/cache behavior and the configured credential deadlines.
Account concurrency bounds the number of separately configured role providers.
Do not interpret an executor limit as a precise account-wide cap on all STS
credential traffic.

`AssumeRoleOptions` is not serialized. Its `Debug` implementation retains only
whether an external ID is present, and discovery reports contain no access
keys, secret keys, session tokens, or external IDs. This module does not offer
interactive MFA prompting; use a suitable existing profile/credential source.

## Admission, pagination, cancellation, and resource bounds

Every explicit discovery request, including caller `GetCallerIdentity`, uses
the shared `ApiExecutor`. The SDK is configured for one inventory attempt;
the executor owns admitted retries, backoff, and limits. Discovery adds no
retry loop around the executor.

Organizations uses the bucket
`{partition}:{account}:global:organizations:discovery`. Config and Resource
Explorer use
`{partition}:{account}:{region}:{service}:discovery`.
Bootstrap identity uses `identity:{region}:sts`, matching the existing scan
bootstrap convention. These are pacing policy keys, not a claim that every AWS
quota has the same scope. Configure the existing executor policy to match the
actual quotas and account workload.

No discovery code spawns tasks or holds a lock across an await. The report,
cursor, continuation token, and item budget have one owner. At most one page
is mapped for each active listing. Fixed `join!` branches allow Organizations,
Config, and Explorer progress under the shared global gate; hierarchy expands
only two child listings for its current parent.

| Bound | Limit and behavior |
| --- | --- |
| Pages per individual listing | Requested value in `1..=10,000`; a remaining token at the limit makes coverage partial. |
| Caller page-size request | `1..=1,000`; adapters clamp to each API's smaller maximum. |
| Raw items in one response | At most 1,000 before mapping. |
| Organization accounts/memberships | At most 100,000 in each result category. |
| Organization roots | At most 32. |
| Organizational units | At most 10,000, depth at most 32; at most 10,032 parent visits. |
| Config aggregators / Explorer views | At most 1,000 per listing. |
| Continuation token | At most 100,000 bytes, consistent with Organizations; repeated token hashes stop traversal. |
| Config source metadata | At most 16 account source groups, 10,000 total declared account IDs, 64 regions per regional list. |
| Explorer view properties | At most 64 included property names. |
| Failure records | At most 1,024 per coverage result, each message truncated to 2,048 characters; excess remains counted. |

Duplicate item identities are rejected with partial evidence. Token hashes are
retained for cycle detection; raw tokens are not added to reports. A response
page that would exceed the remaining item budget is rejected as a whole;
previous pages survive. Invalid items mark coverage partial while valid sibling
items are retained.

Cancellation is checked before configuration loading and before each new page
or hierarchy parent. Explicit AWS operations are selected against executor
cancellation. A cancelled traversal returns retained observations and partial
coverage. Cancellation before caller identity has been established returns an
error, because an attributed discovery report cannot yet be constructed.

These are logical result/operation bounds. They do not impose a fixed process
RSS limit on SDK response deserialization or third-party HTTP buffers.

## IAM and endpoint prerequisites

The implementation sends the following read operations when their features are
requested. This is an operation manifest, not an automatically attached policy:

| Purpose | API / IAM action |
| --- | --- |
| Bootstrap identity | `sts:GetCallerIdentity` |
| Account listing | `organizations:ListAccounts` |
| Root/OU hierarchy | `organizations:ListRoots`, `organizations:ListOrganizationalUnitsForParent`, `organizations:ListChildren` |
| Existing Config sources | `config:DescribeConfigurationAggregators` |
| Existing Explorer sources | `resource-explorer-2:GetIndex`, `resource-explorer-2:ListViews`, `resource-explorer-2:GetDefaultView`, `resource-explorer-2:GetView` |
| Explicit cross-account access | `sts:AssumeRole` for the selected existing role |

Organizations enumeration must run from a management or appropriately delegated
administrator account, as documented by the Organizations APIs. Permission to
list organization accounts does not grant access to member-account resources.
Collector IAM operations such as `config:SelectAggregateResourceConfig` and
`resource-explorer-2:Search` are additional to this discovery manifest.

No-source and permission-denied outcomes remain distinct. The adapter does not
respond to either by changing IAM, enabling services, or provisioning sources.

## Verification scope

All 28 discovery unit tests passed in the complete `inventory-aws` library test
run on 2026-11-12. The executed suite output is retained in
`artifacts/aws-tests-v04.log`; subsequent combined gates are recorded in
`VERIFICATION.md`. Owned discovery sources also passed `rustfmt --check`.

The owned unit tests use SDK builders and injected page functions. They cover
ACTIVE-only eligibility, omitted/unknown state, malformed identities, hierarchy
duplicates/depth, empty-page continuation, token cycles, page/item limits,
partial results after denial, cancellation before/between pages, bounded
diagnostics, source ambiguity/default consistency, scope validation, and filter
redaction. Credential tests construct a lazy provider from synthetic static
test credentials and test timeout validation; they never fetch credentials or
call AWS.

Live organization access, trust-policy authorization, eventual consistency,
SDK credential refresh against STS, and production throughput require an
authorized AWS environment. These are not established by the synthetic tests.
The full asynchronous skill verification remains subject to the unavailable
reference/gate limitations recorded in `RUST_REVIEW.md`; local test success is
not a formal structural-correctness certification.

[accounts]: https://docs.aws.amazon.com/organizations/latest/APIReference/API_ListAccounts.html
[units]: https://docs.aws.amazon.com/organizations/latest/APIReference/API_ListOrganizationalUnitsForParent.html
[children]: https://docs.aws.amazon.com/organizations/latest/APIReference/API_ListChildren.html
[config]: https://docs.aws.amazon.com/config/latest/APIReference/API_DescribeConfigurationAggregators.html
[index]: https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetIndex.html
[default-view]: https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetDefaultView.html
[view]: https://docs.aws.amazon.com/resource-explorer/latest/apireference/API_GetView.html
[assume-role]: https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html

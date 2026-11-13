//! Plan JSON assembly. Every section is a pure function of its inputs.
use super::*;

pub(super) fn plan(
    options: &PlanArgs,
    catalog: build::CatalogPlan,
    targets: &accounting::Targets,
    requests: &accounting::Requests,
) -> Result<Value> {
    Ok(json!({
        "plan_version":3, "execution":"offline planning only",
        "targets":targets_plan(options, targets),
        "coverage":coverage(&catalog)?,
        "catalog_evidence":catalog.evidence, "parent_catalog_evidence":catalog.parent_evidence,
        "stages":stages(),
        "direct":catalog.native, "cloud_control":catalog.cloud,
        "supplemental":supplemental(options),
        "request_accounting":request_accounting(requests),
        "execution_bounds":execution_bounds(options),
        "architecture_semantics":architecture_semantics()
    }))
}

fn targets_plan(options: &PlanArgs, targets: &accounting::Targets) -> Value {
    json!({
        "profiles":targets.profiles, "profile_count":targets.profile_count,
        "default_credential_chain":options.profile.is_empty(),
        "regions":targets.regions,
        "region_selection":if options.region.is_empty() { "Discover opted-in regions with EC2 DescribeRegions when a regional collector is selected" } else { "Use exactly the selected regional endpoints" },
        "account_identity":"Resolved with STS GetCallerIdentity at execution; profiles can refer to the same account",
        "organization_role":options.accounts.organization_role, "role_arn":options.accounts.role_arn,
        "account_filters":options.accounts.account_id, "max_accounts":options.accounts.max_accounts,
        "resolved_target_count":targets.target_count,
        "external_id_environment_variable":options.accounts.external_id_env,
        "organization_selection":"Complete Organizations ListAccounts traversal; ACTIVE accounts only; explicit account filters must match eligible discovered accounts; bound overflow fails before scans"
    })
}

fn coverage(catalog: &build::CatalogPlan) -> Result<Value> {
    let mut services = BTreeMap::<String, usize>::new();
    for entry in &catalog.cloud {
        let value = services
            .entry(string(entry, "service")?.to_owned())
            .or_default();
        *value = value.checked_add(1).context("service count overflow")?;
    }
    Ok(json!({
        "direct_resource_kinds":catalog.native.len(),
        "cloud_control_resource_types":catalog.cloud.len(),
        "cloud_control_root_types":catalog.root_types,
        "cloud_control_parent_dependent_types":catalog.child_types,
        "implicit_parent_types":catalog.implicit_parents,
        "cloud_control_types_by_service":services, "live_validated":false,
        "configured_list_read_support":"Pinned official us-east-1 resource schemas with list/read handlers; runtime region, permissions and provider failures remain explicit"
    }))
}

fn stages() -> Value {
    json!([
        {"stage":"target discovery", "operations":["Organizations ListAccounts when organization-role is selected"], "dependency":"existing management/delegated source session; complete bounded account listing"},
        {"stage":"identity", "operations":["STS GetCallerIdentity"], "dependency":"existing credential chain; one profile job"},
        {"stage":"global discovery", "dependency":"resolved account identity", "scope":"S3, IAM, Route53 and CloudFront once per target scan outside regional fan-out"},
        {"stage":"regional discovery", "dependency":"identity and selected/discovered regions", "pagination":"One cursor chain is sequential; independent types and scopes overlap"},
        {"stage":"parent scoped discovery", "dependency":"successful detailed reads of reviewed roots", "scope":"account + region + child type + exact parent identity", "pagination":"Each parent has an independent bounded cursor chain; failed parent discovery never proves absence of children"},
        {"stage":"resource enrichment", "dependency":"identifiers returned by discovery", "bounded_fanout":4, "failure_scope":"A denied or malformed resource does not erase successful siblings"},
        {"stage":"storage", "dependency":"validated complete page", "commit":"SQLx transaction: resources, edges, fingerprint and continuation checkpoint"},
        {"stage":"architecture projection", "dependency":"finalized inventory snapshot", "operations":["architecture", "graph-export", "graph-push"]}
    ])
}

fn supplemental(options: &PlanArgs) -> Vec<Value> {
    let mut supplemental = Vec::new();
    supplemental.extend(config_source(options));
    supplemental.extend(explorer_source(options));
    supplemental.extend(discovery_source(options));
    supplemental
}

fn config_source(options: &PlanArgs) -> Option<Value> {
    let aggregator = options.config_aggregator.as_ref()?;
    Some(json!({
        "source":"aws_config", "existing_aggregator":aggregator,
        "operation":"SelectAggregateResourceConfig", "quota_group":"config:read",
        "coverage":"Only resources recorded and aggregated by the existing Config setup; no native-equivalent relationship guarantee."
    }))
}

fn explorer_source(options: &PlanArgs) -> Option<Value> {
    let view = options.resource_explorer_view.as_ref()?;
    Some(json!({
        "source":"resource_explorer", "existing_view":view,
        "operation":"ListResources", "quota_group":"resource-explorer-2:read",
        "coverage":"Only resources visible through this existing regional view and index."
    }))
}

fn discovery_source(options: &PlanArgs) -> Option<Value> {
    if !options.discover_sources {
        return None;
    }
    Some(json!({
        "source":"automatic_discovery", "scope":"each selected account and region",
        "config":options.config_aggregator.is_none(), "resource_explorer":options.resource_explorer_view.is_none(),
        "selection":"Use an existing unambiguous aggregator or validated default/single view; absent is recorded, ambiguous/unavailable is partial coverage",
        "creates_infrastructure":false
    }))
}

fn request_accounting(requests: &accounting::Requests) -> Value {
    json!({
        "first_cloud_control_list_requests_if_all_scopes_reached":requests.list,
        "first_root_list_requests_if_all_scopes_reached":requests.root_list,
        "child_list_requests":requests.child,
        "formula":"Root ListResources pages per root type/account/region; child ListResources pages per reviewed discovered parent; one GetResource per identifier; retries add attempts",
        "total_requests":null, "reason":"Page counts, discovered identifiers, failures and retries are not known before collection",
        "quota_key":"partition:account:region:service:operation_group",
        "default_rate":"5 request starts per second per bucket; configured ceiling, not an assertion of the account quota",
        "retry_owner":"ApiExecutor; service SDK max_attempts=1 so each explicit inventory SDK attempt is admitted and counted; credential-provider traffic and exact on-wire timing are outside this counter",
        "max_attempts_per_operation_default":5,
        "provider_quotas":"Cloud Control's resource handler also calls the underlying AWS service; its quotas can throttle independently"
    })
}

fn execution_bounds(options: &PlanArgs) -> Value {
    json!({
        "profiles_in_flight":4, "regions_per_profile_in_flight":4,
        "max_target_accounts":options.accounts.max_accounts,"hard_target_account_cap":1024,
        "default_http_attempts_in_flight":128, "hard_http_attempt_cap":4096,
        "default_attempts_per_bucket_in_flight":4,
        "cloud_control_root_groups_per_region_in_flight":4,
        "parent_items_per_root_in_flight":4,
        "child_types_per_parent_in_flight":2,
        "reads_per_parent_type_scope_in_flight":4,
        "max_parent_model_scopes_per_root_group":10000,
        "max_child_identifiers_per_root_group":100000,
        "max_pages_per_scope":10000, "max_items_per_scope":100000,
        "max_persisted_events_per_invocation":1000000,
        "writer_queue_events":4
    })
}

fn architecture_semantics() -> Value {
    json!({
        "nodes":"Canonical ARNs where authoritative; explicitly namespaced identities otherwise",
        "edges":"Observed configuration references with source and field provenance",
        "unresolved_endpoints":"Retained as references; not invented observations",
        "does_not_prove":["effective IAM authorization", "network reachability", "absence of resources after an incomplete scope"]
    })
}

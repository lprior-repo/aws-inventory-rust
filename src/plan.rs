//! Pure collection planning. This module never loads credentials or performs I/O.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, anyhow, ensure};
use serde_json::{Value, json};

use crate::{catalog, cli::PlanArgs};

const CLOUD_CONTROL_CATALOG: &str = include_str!("../catalog/cloudcontrol-resources.json");
pub(crate) const PARENT_CATALOG: &str =
    include_str!("../catalog/cloudcontrol-parent-resources.json");

pub fn create(options: PlanArgs) -> Result<Value> {
    validate_options(&options)?;
    let catalog = full_catalog()?;
    let groups = select_groups(&catalog, &options)?;
    let native: Vec<Value> = array(&catalog, "tables")?
        .iter()
        .filter(|entry| {
            entry
                .get("selector")
                .and_then(Value::as_str)
                .is_some_and(|group| groups.contains(group))
        })
        .map(native_plan)
        .collect::<Result<_>>()?;
    let cloud = catalog
        .get("cloud_control")
        .context("missing Cloud Control catalog")?;
    let mut requested: BTreeSet<&str> = options.resource_type.iter().map(String::as_str).collect();
    let known: BTreeSet<&str> = array(cloud, "resource_types")?
        .iter()
        .map(|entry| string(entry, "type_name"))
        .collect::<Result<_>>()?;
    for name in &requested {
        ensure!(
            known.contains(name),
            "resource type is not in the pinned collection catalog: {name}"
        );
    }
    let implicit_parents: BTreeSet<&str> = if requested.is_empty() {
        BTreeSet::new()
    } else {
        array(cloud, "resource_types")?
            .iter()
            .filter(|entry| {
                entry
                    .get("type_name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| requested.contains(name))
            })
            .filter_map(|entry| {
                entry
                    .pointer("/parent_workflow/parent_type")
                    .and_then(Value::as_str)
            })
            .filter(|name| !requested.contains(name))
            .collect()
    };
    requested.extend(implicit_parents.iter().copied());
    let cloud_plans: Vec<Value> = if options.cloud_control {
        array(cloud, "resource_types")?
            .iter()
            .filter(|entry| {
                entry
                    .get("type_name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| requested.is_empty() || requested.contains(name))
            })
            .map(cloud_plan)
            .collect::<Result<_>>()?
    } else {
        Vec::new()
    };
    let profiles: BTreeSet<&str> = options.profile.iter().map(String::as_str).collect();
    let regions: BTreeSet<&str> = options.region.iter().map(String::as_str).collect();
    let profile_count = profiles.len().max(1);
    let target_count = if options.accounts.organization_role.is_some() {
        // Discovery can return unknown, suspended or inaccessible accounts;
        // no claimed target count is invented by this credential-free planner.
        None
    } else {
        Some(profile_count)
    };
    let root_types = cloud_plans
        .iter()
        .filter(|entry| {
            entry.pointer("/discovery/resource_model_required") == Some(&Value::Bool(false))
        })
        .count();
    let child_types = cloud_plans.len().saturating_sub(root_types);
    let list_requests = if regions.is_empty() || child_types > 0 {
        None
    } else {
        target_count
            .and_then(|targets| cloud_plans.len().checked_mul(targets))
            .and_then(|count| count.checked_mul(regions.len()))
    };
    let root_list_requests = if regions.is_empty() {
        None
    } else {
        target_count
            .and_then(|targets| root_types.checked_mul(targets))
            .and_then(|count| count.checked_mul(regions.len()))
    };
    let mut services = BTreeMap::<String, usize>::new();
    for entry in &cloud_plans {
        let value = services
            .entry(string(entry, "service")?.to_owned())
            .or_default();
        *value = value.checked_add(1).context("service count overflow")?;
    }
    let mut supplemental = Vec::new();
    if let Some(aggregator) = &options.config_aggregator {
        supplemental.push(json!({
            "source":"aws_config", "existing_aggregator":aggregator,
            "operation":"SelectAggregateResourceConfig", "quota_group":"config:read",
            "coverage":"Only resources recorded and aggregated by the existing Config setup; no native-equivalent relationship guarantee."
        }));
    }
    if let Some(view) = &options.resource_explorer_view {
        supplemental.push(json!({
            "source":"resource_explorer", "existing_view":view,
            "operation":"ListResources", "quota_group":"resource-explorer-2:read",
            "coverage":"Only resources visible through this existing regional view and index."
        }));
    }
    if options.discover_sources {
        supplemental.push(json!({
            "source":"automatic_discovery", "scope":"each selected account and region",
            "config":options.config_aggregator.is_none(), "resource_explorer":options.resource_explorer_view.is_none(),
            "selection":"Use an existing unambiguous aggregator or validated default/single view; absent is recorded, ambiguous/unavailable is partial coverage",
            "creates_infrastructure":false
        }));
    }
    Ok(json!({
        "plan_version":3, "execution":"offline planning only", "catalog_evidence":cloud.get("evidence"),
        "parent_catalog_evidence":cloud.get("parent_evidence"),
        "targets":{
            "profiles":profiles, "profile_count":profile_count,
            "default_credential_chain":options.profile.is_empty(),
            "regions":regions,
            "region_selection":if options.region.is_empty() { "Discover opted-in regions with EC2 DescribeRegions when a regional collector is selected" } else { "Use exactly the selected regional endpoints" },
            "account_identity":"Resolved with STS GetCallerIdentity at execution; profiles can refer to the same account",
            "organization_role":options.accounts.organization_role, "role_arn":options.accounts.role_arn,
            "account_filters":options.accounts.account_id, "max_accounts":options.accounts.max_accounts,
            "resolved_target_count":target_count,
            "external_id_environment_variable":options.accounts.external_id_env,
            "organization_selection":"Complete Organizations ListAccounts traversal; ACTIVE accounts only; explicit account filters must match eligible discovered accounts; bound overflow fails before scans"
        },
        "coverage":{
            "direct_resource_kinds":native.len(), "cloud_control_resource_types":cloud_plans.len(),
            "cloud_control_root_types":root_types,"cloud_control_parent_dependent_types":child_types,
            "implicit_parent_types":implicit_parents,
            "cloud_control_types_by_service":services, "live_validated":false,
            "configured_list_read_support":"Pinned official us-east-1 resource schemas with list/read handlers; runtime region, permissions and provider failures remain explicit"
        },
        "stages":[
            {"stage":"target discovery", "operations":["Organizations ListAccounts when organization-role is selected"], "dependency":"existing management/delegated source session; complete bounded account listing"},
            {"stage":"identity", "operations":["STS GetCallerIdentity"], "dependency":"existing credential chain; one profile job"},
            {"stage":"global discovery", "dependency":"resolved account identity", "scope":"S3, IAM, Route53 and CloudFront once per target scan outside regional fan-out"},
            {"stage":"regional discovery", "dependency":"identity and selected/discovered regions", "pagination":"One cursor chain is sequential; independent types and scopes overlap"},
            {"stage":"parent scoped discovery", "dependency":"successful detailed reads of reviewed roots", "scope":"account + region + child type + exact parent identity", "pagination":"Each parent has an independent bounded cursor chain; failed parent discovery never proves absence of children"},
            {"stage":"resource enrichment", "dependency":"identifiers returned by discovery", "bounded_fanout":4, "failure_scope":"A denied or malformed resource does not erase successful siblings"},
            {"stage":"storage", "dependency":"validated complete page", "commit":"SQLx transaction: resources, edges, fingerprint and continuation checkpoint"},
            {"stage":"architecture projection", "dependency":"finalized inventory snapshot", "operations":["architecture", "graph-export", "graph-push"]}
        ],
        "direct":native, "cloud_control":cloud_plans, "supplemental":supplemental,
        "request_accounting":{
            "first_cloud_control_list_requests_if_all_scopes_reached":list_requests,
            "first_root_list_requests_if_all_scopes_reached":root_list_requests,
            "child_list_requests":if child_types > 0 { Value::Null } else { json!(0) },
            "formula":"Root ListResources pages per root type/account/region; child ListResources pages per reviewed discovered parent; one GetResource per identifier; retries add attempts",
            "total_requests":null, "reason":"Page counts, discovered identifiers, failures and retries are not known before collection",
            "quota_key":"partition:account:region:service:operation_group",
            "default_rate":"5 request starts per second per bucket; configured ceiling, not an assertion of the account quota",
            "retry_owner":"ApiExecutor; service SDK max_attempts=1 so each explicit inventory SDK attempt is admitted and counted; credential-provider traffic and exact on-wire timing are outside this counter",
            "max_attempts_per_operation_default":5,
            "provider_quotas":"Cloud Control's resource handler also calls the underlying AWS service; its quotas can throttle independently"
        },
        "execution_bounds":{
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
        },
        "architecture_semantics":{
            "nodes":"Canonical ARNs where authoritative; explicitly namespaced identities otherwise",
            "edges":"Observed configuration references with source and field provenance",
            "unresolved_endpoints":"Retained as references; not invented observations",
            "does_not_prove":["effective IAM authorization", "network reachability", "absence of resources after an incomplete scope"]
        }
    }))
}

#[cfg(test)]
mod tests;

mod descriptors;
pub use descriptors::full_catalog;

mod selection;
use selection::array;
use selection::cloud_plan;
use selection::native_plan;
use selection::select_groups;
use selection::string;
use selection::validate_options;

//! Pure collection planning. This module never loads credentials or performs I/O.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, anyhow, ensure};
use serde_json::{Value, json};

use crate::{catalog, cli::PlanArgs};

const CLOUD_CONTROL_CATALOG: &str = include_str!("../catalog/cloudcontrol-resources.json");
pub(crate) const PARENT_CATALOG: &str =
    include_str!("../catalog/cloudcontrol-parent-resources.json");

fn cloud_catalog() -> Result<Value> {
    let mut catalog: Value = serde_json::from_str(CLOUD_CONTROL_CATALOG)?;
    let parents: Value = serde_json::from_str(PARENT_CATALOG)?;
    let parent_types = array(&parents, "resource_types")?;
    let parent_plans = array(&parents, "parent_plans")?;
    ensure!(
        parent_types.len() == 21 && parent_plans.len() == parent_types.len(),
        "invalid parent catalog size"
    );
    let root_names: BTreeSet<String> = array(&catalog, "resource_types")?
        .iter()
        .map(|row| string(row, "type_name").map(str::to_owned))
        .collect::<Result<_>>()?;
    let rows = catalog
        .get_mut("resource_types")
        .and_then(Value::as_array_mut)
        .context("missing resource types")?;
    for entry in parent_types {
        let name = string(entry, "type_name")?;
        let matching: Vec<&Value> = parent_plans
            .iter()
            .filter(|plan| plan.get("type_name").and_then(Value::as_str) == Some(name))
            .collect();
        let [plan] = matching.as_slice() else {
            return Err(anyhow!("parent type needs exactly one workflow"));
        };
        ensure!(
            root_names.contains(string(plan, "parent_type")?)
                && entry.get("list_requires_model") == Some(&Value::Bool(true)),
            "unreviewed or nested parent workflow"
        );
        let mut entry = entry.clone();
        entry
            .as_object_mut()
            .context("invalid parent descriptor")?
            .insert("parent_workflow".into(), (*plan).clone());
        rows.push(entry);
    }
    catalog
        .as_object_mut()
        .context("invalid cloud catalog")?
        .insert(
            "parent_evidence".into(),
            parents.get("evidence").cloned().unwrap_or(Value::Null),
        );
    let types = array(&catalog, "resource_types")?;
    ensure!(
        !types.is_empty() && types.len() <= 512,
        "invalid catalog size"
    );
    let mut names = BTreeSet::new();
    let mut views = BTreeSet::new();
    for entry in types {
        ensure!(
            names.insert(string(entry, "type_name")?),
            "duplicate resource type"
        );
        ensure!(
            views.insert(string(entry, "view_name")?),
            "duplicate SQL view"
        );
        ensure!(
            entry.get("read_supported") == Some(&Value::Bool(true)),
            "missing read handler"
        );
        ensure!(
            entry.get("list_supported") == Some(&Value::Bool(true)),
            "missing list handler"
        );
        ensure!(
            entry.get("list_requires_model") == Some(&Value::Bool(false))
                || entry.get("parent_workflow").is_some(),
            "parent discovery workflow missing"
        );
        ensure!(
            string(entry, "scope")? == "regional",
            "unplanned global resource type"
        );
    }
    Ok(catalog)
}

pub fn full_catalog() -> Result<Value> {
    let mut native = catalog::catalog();
    let cloud = cloud_catalog()?;
    let types = array(&cloud, "resource_types")?;
    let mut kinds: BTreeSet<String> = array(&native, "tables")?
        .iter()
        .map(|entry| string(entry, "table").map(str::to_owned))
        .collect::<Result<_>>()?;
    for entry in types {
        kinds.insert(string(entry, "kind")?.to_owned());
    }
    let count = types.len();
    let object = native.as_object_mut().context("invalid native catalog")?;
    object.insert("cloud_control_resource_type_count".into(), json!(count));
    object.insert("distinct_resource_kind_count".into(), json!(kinds.len()));
    object.insert("cloud_control".into(), cloud);
    object.insert(
        "source_preference".into(),
        json!(["direct", "cloud_control", "aws_config", "resource_explorer"]),
    );
    Ok(native)
}

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

fn validate_options(options: &PlanArgs) -> Result<()> {
    crate::targets::validate(&options.accounts, &options.profile)?;
    ensure!(
        options.profile.len() <= 128 && options.region.len() <= 64,
        "profile or region bound exceeded"
    );
    ensure!(
        options.service.len() <= 32 && options.resource_type.len() <= 418,
        "selector bound exceeded"
    );
    ensure!(
        options.cloud_control || options.resource_type.is_empty(),
        "--resource-type requires --cloud-control"
    );
    for profile in &options.profile {
        ensure!(
            !profile.is_empty() && profile.len() <= 2048 && !profile.contains('\0'),
            "invalid profile"
        );
    }
    for region in &options.region {
        ensure!(
            !region.is_empty()
                && region.len() <= 64
                && region.bytes().all(|value| value.is_ascii_lowercase()
                    || value.is_ascii_digit()
                    || value == b'-'),
            "invalid region"
        );
    }
    for value in [&options.config_aggregator, &options.resource_explorer_view]
        .into_iter()
        .flatten()
    {
        ensure!(
            !value.is_empty() && value.len() <= 2048 && !value.contains('\0'),
            "invalid supplemental source"
        );
    }
    Ok(())
}

fn select_groups(catalog: &Value, options: &PlanArgs) -> Result<BTreeSet<String>> {
    let mut groups = BTreeSet::new();
    let selectors = array(catalog, "selectors")?;
    if options.service.is_empty() || options.service.iter().any(|value| value == "all") {
        for selector in selectors {
            for group in array(selector, "collector_groups")? {
                groups.insert(group.as_str().context("invalid group")?.to_owned());
            }
        }
    }
    for selected in &options.service {
        if selected == "all"
            || (selected == "cloud-control" && options.cloud_control)
            || (selected == "config"
                && (options.config_aggregator.is_some() || options.discover_sources))
            || (selected == "resource-explorer"
                && (options.resource_explorer_view.is_some() || options.discover_sources))
        {
            continue;
        }
        let selector = selectors
            .iter()
            .find(|entry| {
                entry.get("selector").and_then(Value::as_str) == Some(selected)
                    || entry
                        .get("aliases")
                        .and_then(Value::as_array)
                        .is_some_and(|aliases| {
                            aliases.iter().any(|alias| alias.as_str() == Some(selected))
                        })
            })
            .ok_or_else(|| {
                anyhow!("unsupported service selector or missing supplemental source: {selected}")
            })?;
        for group in array(selector, "collector_groups")? {
            groups.insert(group.as_str().context("invalid group")?.to_owned());
        }
    }
    Ok(groups)
}

fn native_plan(entry: &Value) -> Result<Value> {
    let service = string(entry, "service")?;
    let group = if matches!(service, "lambda" | "dynamodb") {
        "control_read"
    } else {
        "read"
    };
    Ok(json!({
        "kind":entry.get("table"), "service":service, "selector":entry.get("selector"),
        "operations":entry.get("operations"), "scope":entry.get("scope"),
        "quota_group":format!("{service}:{group}"),
        "relationships":entry.get("outgoing_relationships"),
        "coverage":entry.get("level")
    }))
}

fn cloud_plan(entry: &Value) -> Result<Value> {
    let permissions: BTreeSet<&str> = ["list_permissions", "read_permissions"]
        .into_iter()
        .map(|field| array(entry, field))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|permission| permission.as_str().context("invalid handler permission"))
        .collect::<Result<_>>()?;
    let review: Vec<&str> = permissions
        .into_iter()
        .filter(|permission| {
            let action = permission
                .split_once(':')
                .map_or(*permission, |(_, action)| action)
                .to_ascii_lowercase();
            ![
                "get", "list", "describe", "search", "head", "batchget", "lookup", "select",
                "check", "scan",
            ]
            .iter()
            .any(|prefix| action.starts_with(prefix))
        })
        .collect();
    Ok(json!({
        "type_name":string(entry,"type_name")?, "kind":string(entry,"kind")?,
        "view":string(entry,"view_name")?, "service":string(entry,"service")?,
        "scope":if entry.get("parent_workflow").is_some() { "account + region + resource type + exact parent identity" } else { "account + region + resource type" },
        "discovery":{"operation":"CloudControl ListResources", "pagination":"NextToken", "resource_model_required":entry.get("list_requires_model")},
        "parent_workflow":entry.get("parent_workflow"),
        "enrichment":{"operation":"CloudControl GetResource", "parent":"type name + exact listed identifier", "concurrency":4},
        "primary_identifier":entry.get("primary_identifier"), "relationships":entry.get("relationships"),
        "permissions":{
            "gateway":["cloudformation:ListResources", "cloudformation:GetResource"],
            "schema_declared_provider_list":entry.get("list_permissions"),
            "schema_declared_provider_read":entry.get("read_permissions"),
            "declarations_outside_common_read_name_prefixes":review,
            "interpretation":"Provider declarations can be broader than exercised calls; this is not an automatically deployable least-privilege policy or proof that a handler mutates resources",
            "role":"existing caller session; this scanner does not create or pass a new execution role"
        },
        "schema_sha256":entry.get("schema_sha256"), "live_validated":false
    }))
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("invalid catalog array {field}"))
}

fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("invalid catalog string {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> PlanArgs {
        PlanArgs {
            profile: vec!["one".into(), "two".into()],
            region: vec!["us-east-1".into(), "us-west-2".into()],
            service: vec!["all".into()],
            cloud_control: true,
            resource_type: Vec::new(),
            config_aggregator: None,
            resource_explorer_view: None,
            discover_sources: false,
            accounts: crate::cli::AccountArgs::default(),
        }
    }

    #[test]
    fn full_plan_has_schema_backed_breadth_and_explicit_cost_uncertainty() -> Result<()> {
        let plan = create(options())?;
        assert_eq!(
            plan.pointer("/coverage/direct_resource_kinds")
                .and_then(Value::as_u64),
            Some(32)
        );
        let count = plan
            .pointer("/coverage/cloud_control_resource_types")
            .and_then(Value::as_u64)
            .context("missing count")?;
        assert_eq!(count, 418);
        assert_eq!(
            plan.pointer(
                "/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"
            ),
            Some(&Value::Null)
        );
        assert_eq!(
            plan.pointer("/request_accounting/first_root_list_requests_if_all_scopes_reached"),
            Some(&json!(1588))
        );
        assert_eq!(
            plan.pointer("/request_accounting/total_requests"),
            Some(&Value::Null)
        );
        assert_eq!(
            plan.pointer("/coverage/live_validated"),
            Some(&Value::Bool(false))
        );
        Ok(())
    }

    #[test]
    fn selected_children_share_one_explicit_parent_prerequisite() -> Result<()> {
        let mut selected = options();
        selected.resource_type = vec![
            "AWS::EKS::Addon".into(),
            "AWS::EKS::Nodegroup".into(),
            "AWS::EKS::Addon".into(),
        ];
        let plan = create(selected)?;
        assert_eq!(
            plan.pointer("/coverage/cloud_control_resource_types"),
            Some(&json!(3))
        );
        assert_eq!(
            plan.pointer("/coverage/cloud_control_root_types"),
            Some(&json!(1))
        );
        assert_eq!(
            plan.pointer("/coverage/cloud_control_parent_dependent_types"),
            Some(&json!(2))
        );
        assert_eq!(
            plan.pointer("/coverage/implicit_parent_types"),
            Some(&json!(["AWS::EKS::Cluster"]))
        );
        assert_eq!(
            plan.pointer("/request_accounting/first_root_list_requests_if_all_scopes_reached"),
            Some(&json!(4))
        );
        assert_eq!(
            plan.pointer("/request_accounting/child_list_requests"),
            Some(&Value::Null)
        );
        let addon = array(&plan, "cloud_control")?
            .iter()
            .find(|row| row.get("type_name") == Some(&json!("AWS::EKS::Addon")))
            .context("addon plan")?;
        assert_eq!(
            addon.pointer("/discovery/resource_model_required"),
            Some(&json!(true))
        );
        assert_eq!(
            addon.pointer("/parent_workflow/bindings"),
            Some(&json!([{"source_pointer":"/Name","model_property":"ClusterName"}]))
        );
        Ok(())
    }

    #[test]
    fn root_only_request_count_keeps_the_old_exact_contract() -> Result<()> {
        let mut selected = options();
        selected.resource_type = vec!["AWS::EKS::Cluster".into()];
        let plan = create(selected)?;
        assert_eq!(
            plan.pointer("/coverage/implicit_parent_types"),
            Some(&json!([]))
        );
        assert_eq!(
            plan.pointer(
                "/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"
            ),
            Some(&json!(4))
        );
        assert_eq!(
            plan.pointer("/request_accounting/child_list_requests"),
            Some(&json!(0))
        );
        Ok(())
    }

    #[test]
    fn selectors_deduplicate_and_unknown_types_are_errors() -> Result<()> {
        let mut selected = options();
        selected.service = vec!["ec2".into(), "network".into(), "vpc".into()];
        selected.cloud_control = false;
        assert_eq!(
            create(selected)?
                .pointer("/coverage/direct_resource_kinds")
                .and_then(Value::as_u64),
            Some(12)
        );
        let mut invalid = options();
        invalid.resource_type = vec!["AWS::Invented::Missing".into()];
        assert!(create(invalid).is_err());
        let mut invalid = options();
        invalid.service = vec!["all".into(), "unknown".into()];
        assert!(create(invalid).is_err());
        Ok(())
    }

    #[test]
    fn supplemental_only_and_unknown_region_count_remain_explicit() -> Result<()> {
        let mut selected = options();
        selected.service = vec!["config".into()];
        selected.config_aggregator = Some("existing".into());
        selected.cloud_control = false;
        let plan = create(selected)?;
        assert_eq!(
            plan.pointer("/coverage/direct_resource_kinds")
                .and_then(Value::as_u64),
            Some(0)
        );
        assert_eq!(array(&plan, "supplemental")?.len(), 1);
        let mut dynamic = options();
        dynamic.region.clear();
        assert_eq!(
            create(dynamic)?.pointer(
                "/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"
            ),
            Some(&Value::Null)
        );
        Ok(())
    }

    #[test]
    fn global_and_organization_plans_keep_account_count_unknown_until_discovery() -> Result<()> {
        let mut selected = options();
        selected.profile = vec!["management".into()];
        selected.service = vec![
            "s3".into(),
            "iam".into(),
            "route53".into(),
            "cloudfront".into(),
        ];
        selected.discover_sources = true;
        selected.accounts.organization_role = Some("audit/InventoryReader".into());
        selected.accounts.external_id_env = Some("NOT_LOADED_DURING_PLAN".into());
        let plan = create(selected)?;
        assert_eq!(
            plan.pointer("/coverage/direct_resource_kinds"),
            Some(&json!(7))
        );
        assert_eq!(
            plan.pointer("/targets/resolved_target_count"),
            Some(&Value::Null)
        );
        assert_eq!(
            plan.pointer(
                "/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"
            ),
            Some(&Value::Null)
        );
        assert_eq!(
            plan.pointer("/supplemental/0/creates_infrastructure"),
            Some(&json!(false))
        );
        Ok(())
    }

    #[test]
    fn source_discovery_allows_source_selectors_without_loading_credentials() -> Result<()> {
        let mut selected = options();
        selected.service = vec!["config".into(), "resource-explorer".into()];
        selected.cloud_control = false;
        selected.discover_sources = true;
        let plan = create(selected)?;
        assert_eq!(
            plan.pointer("/coverage/direct_resource_kinds"),
            Some(&json!(0))
        );
        assert_eq!(array(&plan, "supplemental")?.len(), 1);
        Ok(())
    }
}

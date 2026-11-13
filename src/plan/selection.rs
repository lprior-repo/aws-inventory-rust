use super::*;

pub(super) fn validate_options(options: &PlanArgs) -> Result<()> {
    crate::targets::validate(&options.accounts, &options.profile)?;
    validate_bounds(options)?;
    ensure!(
        options
            .profile
            .iter()
            .all(|profile| valid_text(profile, 2048)),
        "invalid profile"
    );
    for region in &options.region {
        ensure!(valid_region(region), "invalid region");
    }
    ensure!(
        [&options.config_aggregator, &options.resource_explorer_view]
            .into_iter()
            .flatten()
            .all(|value| valid_text(value, 2048)),
        "invalid supplemental source"
    );
    Ok(())
}

fn validate_bounds(options: &PlanArgs) -> Result<()> {
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
    Ok(())
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.contains('\0')
}

fn valid_region(region: &str) -> bool {
    !region.is_empty()
        && region.len() <= 64
        && region
            .bytes()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == b'-')
}

pub(super) fn select_groups(catalog: &Value, options: &PlanArgs) -> Result<BTreeSet<String>> {
    let selectors = array(catalog, "selectors")?;
    let mut groups = BTreeSet::new();
    if options.service.is_empty() || options.service.iter().any(|value| value == "all") {
        add_all_groups(selectors, &mut groups)?;
    }
    for selected in &options.service {
        if supplemented(selected, options) {
            continue;
        }
        add_groups(selector_for(selectors, selected)?, &mut groups)?;
    }
    Ok(groups)
}

fn add_all_groups(selectors: &[Value], groups: &mut BTreeSet<String>) -> Result<()> {
    for selector in selectors {
        add_groups(selector, groups)?;
    }
    Ok(())
}

fn add_groups(selector: &Value, groups: &mut BTreeSet<String>) -> Result<()> {
    for group in array(selector, "collector_groups")? {
        groups.insert(group.as_str().context("invalid group")?.to_owned());
    }
    Ok(())
}

/// Selectors whose work is already covered by a supplemental source.
fn supplemented(selected: &str, options: &PlanArgs) -> bool {
    selected == "all"
        || (selected == "cloud-control" && options.cloud_control)
        || (selected == "config"
            && (options.config_aggregator.is_some() || options.discover_sources))
        || (selected == "resource-explorer"
            && (options.resource_explorer_view.is_some() || options.discover_sources))
}

fn selector_for<'a>(selectors: &'a [Value], selected: &str) -> Result<&'a Value> {
    selectors
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
        })
}

pub(super) fn native_plan(entry: &Value) -> Result<Value> {
    let service = string(entry, "service")?;
    let group = if matches!(
        service,
        "lambda" | "dynamodb" | "sqs" | "sns" | "kms" | "stepfunctions"
    ) {
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

/// Actions whose names already imply a read-only provider call.
const COMMON_PREFIXES: [&str; 10] = [
    "get", "list", "describe", "search", "head", "batchget", "lookup", "select", "check", "scan",
];

pub(super) fn cloud_plan(entry: &Value) -> Result<Value> {
    let permissions = handler_permissions(entry)?;
    let review = uncommon_permissions(permissions);
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

fn handler_permissions(entry: &Value) -> Result<BTreeSet<&str>> {
    ["list_permissions", "read_permissions"]
        .into_iter()
        .map(|field| array(entry, field))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|permission| permission.as_str().context("invalid handler permission"))
        .collect::<Result<_>>()
}

fn uncommon_permissions(permissions: BTreeSet<&str>) -> Vec<&str> {
    permissions
        .into_iter()
        .filter(|permission| {
            let action = permission
                .split_once(':')
                .map_or(*permission, |(_, action)| action)
                .to_ascii_lowercase();
            !COMMON_PREFIXES
                .iter()
                .any(|prefix| action.starts_with(prefix))
        })
        .collect()
}

pub(super) fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("invalid catalog array {field}"))
}

pub(super) fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("invalid catalog string {field}"))
}

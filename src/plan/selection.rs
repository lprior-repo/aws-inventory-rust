use super::*;

pub(super) fn validate_options(options: &PlanArgs) -> Result<()> {
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

pub(super) fn select_groups(catalog: &Value, options: &PlanArgs) -> Result<BTreeSet<String>> {
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

pub(super) fn native_plan(entry: &Value) -> Result<Value> {
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

pub(super) fn cloud_plan(entry: &Value) -> Result<Value> {
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

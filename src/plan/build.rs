//! Catalog selection for offline plans.
use super::*;

/// Catalog-derived inputs shared by coverage and request accounting.
pub(super) struct CatalogPlan {
    pub evidence: Option<Value>,
    pub parent_evidence: Option<Value>,
    pub native: Vec<Value>,
    pub cloud: Vec<Value>,
    pub implicit_parents: BTreeSet<String>,
    pub root_types: usize,
    pub child_types: usize,
    pub universal: Option<UniversalPlan>,
}

/// Universal-catalog evidence and its list-scope count.
pub(super) struct UniversalPlan {
    pub list_types: u64,
    pub list_scopes: usize,
    pub catalog: Value,
}

pub(super) fn catalog(options: &PlanArgs, selection: &Selection) -> Result<CatalogPlan> {
    let full = full_catalog()?;
    let groups = select_groups(&full, options)?;
    let cloud = full
        .get("cloud_control")
        .context("missing Cloud Control catalog")?;
    let native = native_plans(&full, &groups)?;
    let universal = universal_plan(selection)?;
    let mut requested = requested_types(cloud, selection)?;
    let implicit_parents = implicit_parents(cloud, &mut requested)?;
    let plans = cloud_plans(options, cloud, &requested)?;
    let root_types = root_type_count(&plans);
    let child_types = plans.len().saturating_sub(root_types);
    Ok(CatalogPlan {
        evidence: cloud.get("evidence").cloned(),
        parent_evidence: cloud.get("parent_evidence").cloned(),
        native,
        cloud: plans,
        implicit_parents,
        root_types,
        child_types,
        universal,
    })
}

fn root_type_count(plans: &[Value]) -> usize {
    plans
        .iter()
        .filter(|entry| {
            entry.pointer("/discovery/resource_model_required") == Some(&Value::Bool(false))
        })
        .count()
}

fn native_plans(catalog: &Value, groups: &BTreeSet<String>) -> Result<Vec<Value>> {
    array(catalog, "tables")?
        .iter()
        .filter(|entry| {
            entry
                .get("selector")
                .and_then(Value::as_str)
                .is_some_and(|group| groups.contains(group))
        })
        .map(native_plan)
        .collect()
}

fn universal_plan(selection: &Selection) -> Result<Option<UniversalPlan>> {
    if !selection.is_universal() {
        return Ok(None);
    }
    let (list_types, catalog) = universal::evidence()?;
    let list_scopes = usize::try_from(list_types).context("universal list type count overflow")?;
    Ok(Some(UniversalPlan {
        list_types,
        list_scopes,
        catalog,
    }))
}

/// Validate explicit names against the pinned catalog, deduplicating them.
fn requested_types(cloud: &Value, selection: &Selection) -> Result<BTreeSet<String>> {
    let known: BTreeSet<&str> = array(cloud, "resource_types")?
        .iter()
        .map(|entry| string(entry, "type_name"))
        .collect::<Result<_>>()?;
    selection
        .types()
        .iter()
        .try_fold(BTreeSet::new(), |mut requested, kind| {
            let name = kind.as_str();
            ensure!(
                known.contains(name),
                "resource type is not in the pinned collection catalog: {name}"
            );
            if !requested.contains(name) {
                requested.insert(name.to_owned());
            }
            Ok(requested)
        })
}

/// Add the parent types selected children require, preserving plan output.
fn implicit_parents(cloud: &Value, requested: &mut BTreeSet<String>) -> Result<BTreeSet<String>> {
    let implicit: BTreeSet<String> = if requested.is_empty() {
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
            .filter(|name| !requested.contains(*name))
            .map(str::to_owned)
            .collect()
    };
    requested.extend(implicit.iter().cloned());
    Ok(implicit)
}

fn cloud_plans(
    options: &PlanArgs,
    cloud: &Value,
    requested: &BTreeSet<String>,
) -> Result<Vec<Value>> {
    if !options.cloud_control {
        return Ok(Vec::new());
    }
    array(cloud, "resource_types")?
        .iter()
        .filter(|entry| {
            entry
                .get("type_name")
                .and_then(Value::as_str)
                .is_some_and(|name| requested.is_empty() || requested.contains(name))
        })
        .map(cloud_plan)
        .collect()
}

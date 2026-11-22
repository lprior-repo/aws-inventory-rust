use super::*;

pub(super) fn cloud_catalog() -> Result<Value> {
    let mut catalog: Value = serde_json::from_str(CLOUD_CONTROL_CATALOG)?;
    let parents: Value = toml::from_str(PARENT_CATALOG)?;
    ensure_parent_size(&parents)?;
    let root_names = root_names(&catalog)?;
    append_parent_rows(&mut catalog, &parents, &root_names)?;
    insert_parent_evidence(&mut catalog, &parents)?;
    validate_types(&catalog)?;
    Ok(catalog)
}

fn ensure_parent_size(parents: &Value) -> Result<()> {
    let parent_types = array(parents, "resource_types")?;
    let parent_plans = array(parents, "parent_plans")?;
    ensure!(
        parent_types.len() == 24 && parent_plans.len() == parent_types.len(),
        "invalid parent catalog size"
    );
    Ok(())
}

fn root_names(catalog: &Value) -> Result<BTreeSet<String>> {
    array(catalog, "resource_types")?
        .iter()
        .map(|row| string(row, "type_name").map(str::to_owned))
        .collect()
}

fn append_parent_rows(
    catalog: &mut Value,
    parents: &Value,
    root_names: &BTreeSet<String>,
) -> Result<()> {
    let parent_types = array(parents, "resource_types")?;
    let parent_plans = array(parents, "parent_plans")?;
    let rows = catalog
        .get_mut("resource_types")
        .and_then(Value::as_array_mut)
        .context("missing resource types")?;
    for entry in parent_types {
        rows.push(parent_row(entry, parent_plans, root_names)?);
    }
    Ok(())
}

fn parent_row(
    entry: &Value,
    parent_plans: &[Value],
    root_names: &BTreeSet<String>,
) -> Result<Value> {
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
    Ok(entry)
}

fn insert_parent_evidence(catalog: &mut Value, parents: &Value) -> Result<()> {
    catalog
        .as_object_mut()
        .context("invalid cloud catalog")?
        .insert(
            "parent_evidence".into(),
            parents.get("evidence").cloned().unwrap_or(Value::Null),
        );
    Ok(())
}

fn validate_types(catalog: &Value) -> Result<()> {
    let types = array(catalog, "resource_types")?;
    ensure!(
        !types.is_empty() && types.len() <= 512,
        "invalid catalog size"
    );
    let mut names = BTreeSet::new();
    let mut views = BTreeSet::new();
    for entry in types {
        check_type(entry, &mut names, &mut views)?;
    }
    Ok(())
}

fn check_type(
    entry: &Value,
    names: &mut BTreeSet<String>,
    views: &mut BTreeSet<String>,
) -> Result<()> {
    let type_name = string(entry, "type_name")?;
    let view_name = string(entry, "view_name")?;
    ensure!(
        names.insert(type_name.to_owned()),
        "duplicate resource type"
    );
    ensure!(views.insert(view_name.to_owned()), "duplicate SQL view");
    ensure!(flag(entry, "read_supported"), "missing read handler");
    ensure!(flag(entry, "list_supported"), "missing list handler");
    ensure!(
        entry.get("list_requires_model") == Some(&Value::Bool(false))
            || entry.get("parent_workflow").is_some(),
        "parent discovery workflow missing"
    );
    let scope = string(entry, "scope")?;
    ensure!(scope == "regional", "unplanned global resource type");
    Ok(())
}

fn flag(entry: &Value, field: &str) -> bool {
    entry.get(field) == Some(&Value::Bool(true))
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

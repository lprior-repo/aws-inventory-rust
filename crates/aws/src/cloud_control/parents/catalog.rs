use super::*;

pub(in super::super) fn read_catalog() -> Result<ParentCatalog, AwsScanError> {
    let catalog: ParentCatalog = serde_json::from_str(PARENT_CATALOG_JSON)
        .map_err(|error| invalid(format!("embedded parent catalog: {error}")))?;
    if catalog.schema_version != 1
        || catalog.resource_types.len() != PARENT_TYPE_COUNT
        || catalog.parent_plans.len() != PARENT_TYPE_COUNT
        || catalog.evidence.global_sensitive_property_names.is_empty()
        || catalog.evidence.global_sensitive_property_names.len() > 128
    {
        return Err(invalid(
            "invalid parent catalog version, count or redaction policy",
        ));
    }
    Ok(catalog)
}

pub(in super::super) fn catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    let mut catalog = read_catalog()?;
    let sensitive = Arc::new(catalog.evidence.global_sensitive_property_names);
    for descriptor in &mut catalog.resource_types {
        validate_descriptor(descriptor, true)?;
        if descriptor.list_model_validation != "reviewed_parent_model" {
            return Err(invalid(
                "parent descriptor lacks reviewed list-model evidence",
            ));
        }
        descriptor.sensitive_names = Arc::clone(&sensitive);
    }
    Ok(catalog.resource_types)
}

pub(in super::super) fn plans() -> Result<Vec<ParentPlan>, AwsScanError> {
    Ok(read_catalog()?.parent_plans)
}

pub(in super::super) fn validate_catalog(types: &[ResourceType]) -> Result<(), AwsScanError> {
    let mut names = BTreeSet::new();
    let mut views = BTreeSet::new();
    if types.len() != TYPE_COUNT {
        return Err(invalid("combined catalog has an unexpected type count"));
    }
    for descriptor in types {
        if !names.insert(descriptor.type_name.as_str())
            || !views.insert(descriptor.view_name.as_str())
        {
            return Err(invalid("combined catalog repeats a type or view"));
        }
    }
    let mut children = BTreeSet::new();
    for plan in plans()? {
        let child = find_type(types, &plan.type_name)?;
        let parent = find_type(types, &plan.parent_type)?;
        if !children.insert(plan.type_name.clone())
            || !child.list_requires_model
            || parent.list_requires_model
            || plan.bindings.is_empty()
            || plan.bindings.len() > MAX_BINDINGS
            || plan.relation != "CHILD_OF"
            || plan.evidence != "reviewed_parent_resource_model_and_matching_read_properties"
        {
            return Err(invalid("invalid or non-root parent dependency"));
        }
        validate_bindings(&plan, parent, child)?;
    }
    if children.len() != PARENT_TYPE_COUNT {
        return Err(invalid("parent catalog has missing plans"));
    }
    Ok(())
}

pub(in super::super) fn validate_bindings(
    plan: &ParentPlan,
    parent: &ResourceType,
    child: &ResourceType,
) -> Result<(), AwsScanError> {
    let mut properties = BTreeSet::new();
    for binding in &plan.bindings {
        validate_pointer(&binding.source_pointer)?;
        let target = format!("/{}", binding.model_property);
        if binding.model_property.is_empty()
            || binding.model_property.len() > 128
            || !binding
                .model_property
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric())
            || !properties.insert(&binding.model_property)
            || !parent.columns.iter().any(|column| {
                column.pointer == binding.source_pointer && column.json_type == "string"
            })
            || !child
                .columns
                .iter()
                .any(|column| column.pointer == target && column.json_type == "string")
        {
            return Err(invalid("parent binding is not a readable string property"));
        }
    }
    Ok(())
}

pub(in super::super) fn find_type<'a>(
    types: &'a [ResourceType],
    name: &str,
) -> Result<&'a ResourceType, AwsScanError> {
    types
        .iter()
        .find(|descriptor| descriptor.type_name == name)
        .ok_or_else(|| invalid(format!("missing selected prerequisite type {name}")))
}

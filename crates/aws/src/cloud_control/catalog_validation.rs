//! catalog validation operations.
use super::*;

pub(super) fn invalid(message: impl Into<String>) -> AwsScanError {
    AwsScanError::InvalidData(message.into())
}

#[derive(Deserialize)]
struct SensitiveCatalog {
    evidence: CatalogEvidence,
}

/// The global property-name policy from the reviewed catalog evidence. It is
/// shared by reviewed descriptors and synthesized universal descriptors, so an
/// unreviewed provider row cannot bypass redaction.
pub fn global_sensitive_names() -> Result<Arc<BTreeSet<String>>, AwsScanError> {
    let catalog: SensitiveCatalog = serde_json::from_str(CATALOG_JSON)
        .map_err(|error| invalid(format!("embedded Cloud Control catalog: {error}")))?;
    let sensitive = catalog.evidence.global_sensitive_property_names;
    if sensitive.is_empty() || sensitive.len() > 128 {
        return Err(invalid("invalid common sensitive-property policy"));
    }
    Ok(Arc::new(sensitive))
}

/// Parse and validate the embedded, pinned catalog, without credentials or I/O.
pub fn root_catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    let mut catalog: Catalog = serde_json::from_str(CATALOG_JSON)
        .map_err(|error| invalid(format!("embedded Cloud Control catalog: {error}")))?;
    if catalog.schema_version != 1 || catalog.resource_types.len() != ROOT_TYPE_COUNT {
        return Err(invalid(
            "unexpected Cloud Control catalog version or type count",
        ));
    }
    let mut types = BTreeSet::new();
    let mut views = BTreeSet::new();
    let sensitive = Arc::new(catalog.evidence.global_sensitive_property_names);
    if sensitive.is_empty() || sensitive.len() > 128 {
        return Err(invalid("invalid common sensitive-property policy"));
    }
    for descriptor in &mut catalog.resource_types {
        validate_descriptor(descriptor, false)?;
        if !types.insert(descriptor.type_name.clone())
            || !views.insert(descriptor.view_name.clone())
        {
            return Err(invalid("duplicate Cloud Control type or SQL view"));
        }
        descriptor.sensitive_names = Arc::clone(&sensitive);
    }
    Ok(catalog.resource_types)
}

/// Reviewed parent-dependent types, kept separate from the frozen root catalog.
pub fn parent_catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    parents::catalog()
}

/// Complete runnable catalog, including types that need a discovered parent.
pub fn catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    let mut descriptors = root_catalog()?;
    descriptors.extend(parent_catalog()?);
    parents::validate_catalog(&descriptors)?;
    Ok(descriptors)
}

/// An empty selection means the whole pinned catalog. Unknown selections fail
/// before credentials are loaded; duplicate selectors are harmlessly deduped.
pub fn validate_selection(selection: &Selection) -> Result<Vec<ResourceType>, AwsScanError> {
    let selected = match selection {
        Selection::Universal => return super::universal::list_descriptors(),
        Selection::Types(selected) => selected,
    };
    if selected.len() > TYPE_COUNT {
        return Err(AwsScanError::InvalidOptions(
            "at most 421 Cloud Control resource types may be selected".into(),
        ));
    }
    let descriptors = catalog()?;
    let mut requested: BTreeSet<&str> = selected.iter().map(|value| value.as_str()).collect();
    for name in &requested {
        if !descriptors
            .iter()
            .any(|descriptor| descriptor.type_name == *name)
        {
            return Err(AwsScanError::InvalidOptions(format!(
                "resource type is not in the pinned Cloud Control catalog: {}",
                name.chars().take(196).collect::<String>()
            )));
        }
    }
    let plans = parents::plans()?;
    if !requested.is_empty() {
        for plan in &plans {
            if requested.contains(plan.type_name.as_str()) {
                requested.insert(plan.parent_type.as_str());
            }
        }
    }
    Ok(descriptors
        .into_iter()
        .filter(|descriptor| {
            requested.is_empty() || requested.contains(descriptor.type_name.as_str())
        })
        .collect())
}

pub(super) fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn validate_pointer(pointer: &str) -> Result<(), AwsScanError> {
    if !pointer.starts_with('/')
        || pointer.len() > 1_024
        || pointer.split('/').count() > MAX_JSON_DEPTH
        || pointer.contains('\0')
    {
        return Err(invalid("invalid or excessive catalog property pointer"));
    }
    Ok(())
}

pub(super) fn validate_descriptor(
    descriptor: &ResourceType,
    parent_required: bool,
) -> Result<(), AwsScanError> {
    validate_identifier(&descriptor.type_name, "Cloud Control type")?;
    if !descriptor.type_name.starts_with("AWS::")
        || descriptor.type_name.len() > 196
        || !safe_name(&descriptor.kind)
        || !safe_name(&descriptor.view_name)
        || !descriptor.view_name.starts_with("aws_cc_")
        || descriptor.scope != "regional"
        || descriptor.home_region.is_some()
        || !descriptor.read_supported
        || !descriptor.list_supported
        || descriptor.list_requires_model != parent_required
        || descriptor.primary_identifier.is_empty()
        || descriptor.primary_identifier.len() > 16
        || descriptor.columns.len() > 200
        || descriptor.relationships.len() > MAX_RELATIONSHIP_RULES
        || descriptor.write_only_properties.len() > 256
        || descriptor.sensitive_properties.len() > 256
        || descriptor.schema_sha256.len() != 64
        || !descriptor
            .schema_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid(format!(
            "invalid Cloud Control descriptor {}",
            descriptor.type_name
        )));
    }
    for pointer in descriptor
        .primary_identifier
        .iter()
        .chain(&descriptor.arn_pointers)
        .chain(&descriptor.name_pointers)
        .chain(&descriptor.write_only_properties)
        .chain(&descriptor.sensitive_properties)
        .chain(descriptor.arn_identifier_pointer.iter())
    {
        validate_pointer(pointer)?;
    }
    let mut columns = BTreeSet::new();
    for column in &descriptor.columns {
        validate_pointer(&column.pointer)?;
        if !safe_name(&column.name) || !columns.insert(&column.name) {
            return Err(invalid("invalid or duplicate provider column"));
        }
    }
    for relation in &descriptor.relationships {
        validate_pointer(&relation.pointer)?;
        validate_identifier(&relation.relation, "relation")?;
        if !safe_name(&relation.target_kind)
            || !matches!(relation.target_format.as_str(), "arn" | "template")
            || (relation.target_format == "template" && relation.arn_template.is_none())
        {
            return Err(invalid("invalid relationship descriptor"));
        }
        for constraint in [
            &relation.arn_service,
            &relation.arn_resource_prefix,
            &relation.identifier_prefix,
        ] {
            if constraint.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > 128 || value.chars().any(char::is_control)
            }) {
                return Err(invalid("invalid relationship identity constraint"));
            }
        }
        if (relation.target_format == "arn" && relation.identifier_prefix.is_some())
            || (relation.target_format == "template"
                && (relation.arn_service.is_some() || relation.arn_resource_prefix.is_some()))
        {
            return Err(invalid(
                "relationship constraint does not match its identity format",
            ));
        }
    }
    Ok(())
}

//! Pure Cypher literals and observation projection rows.
use super::*;

pub(super) fn literal(value: &str) -> Result<String, GraphError> {
    Ok(serde_json::to_string(value)?)
}

pub(super) fn resource_row(scan: &str, r: Resource) -> Result<String, GraphError> {
    let pairs = [
        ("key", format!("{scan}::{}", r.id)),
        ("id", r.id),
        ("scan_id", scan.into()),
        ("kind", r.kind),
        ("account_id", r.account_id),
        ("region", r.region),
        ("name", r.name.unwrap_or_default()),
        ("properties_json", serde_json::to_string(&r.properties)?),
        ("tags_json", serde_json::to_string(&r.tags)?),
        ("source", r.source.as_str().into()),
    ];
    row_map(&pairs, r.observed_at)
}

pub(super) fn edge_row(scan: &str, e: Edge) -> Result<String, GraphError> {
    let pairs = [
        ("from_key", format!("{scan}::{}", e.from_id)),
        ("to_key", format!("{scan}::{}", e.to_id)),
        ("from_id", e.from_id),
        ("to_id", e.to_id),
        ("scan_id", scan.into()),
        ("relation", e.relation),
        ("source", e.source.as_str().into()),
        ("properties_json", serde_json::to_string(&e.properties)?),
    ];
    row_map(&pairs, e.observed_at)
}

pub(super) fn row_map(pairs: &[(&str, String)], observed_at: i64) -> Result<String, GraphError> {
    let fields = pairs
        .iter()
        .map(|(key, value)| Ok(format!("{key}:{}", literal(value)?)))
        .collect::<Result<Vec<_>, GraphError>>()?;
    Ok(format!(
        "{{{},observed_at:{observed_at}}}",
        fields.join(",")
    ))
}

pub(super) fn validate_name(name: &str) -> Result<(), GraphError> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(GraphError::Invalid("graph and scan names allow only ASCII letters, numbers, underscores and hyphens, up to 128 bytes".into()));
    }
    Ok(())
}

//! Pinned universal catalog parsing, accounting, and list-capable selection.
use super::*;

const UNIVERSAL_CATALOG_JSON: &str =
    include_str!("../../../../../catalog/cloudcontrol-universal.json");
const MAX_UNIVERSAL_TYPES: usize = 2_000;
const UNIVERSAL_VIEW_PREFIX: &str = "aws_cc_u_";

mod names;
use names::{normalized_name, short_digest, unique_name};

/// One row of the pinned universal catalog.
#[derive(Clone, Debug, Deserialize)]
pub struct UniversalRow {
    pub type_name: String,
    #[serde(default)]
    pub schema_sha256: String,
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub parent_type: Option<String>,
    #[serde(default)]
    pub exclusion: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UniversalSummary {
    pub total: usize,
    pub list_capable: usize,
    pub parent_required: usize,
    pub unsupported: usize,
}

#[derive(Deserialize)]
struct UniversalCatalog {
    schema_version: u32,
    summary: UniversalSummary,
    resource_types: Vec<UniversalRow>,
}

/// Catalog digest plus the complete, non-silent outcome partition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UniversalReport {
    pub catalog_digest: String,
    pub total: usize,
    pub list_capable: usize,
    pub parent_required: usize,
    pub unsupported: usize,
    pub skipped_parent_required: Vec<String>,
    pub skipped_unsupported: Vec<String>,
}

pub(super) fn catalog() -> Result<(UniversalSummary, Vec<UniversalRow>), AwsScanError> {
    let parsed = parse_catalog()?;
    let summary = parsed.summary;
    let rows = parsed.resource_types;
    check_catalog_size(&rows, &summary)?;
    let counts = count_capabilities(&rows)?;
    check_summary_partition(&counts, &summary, rows.len())?;
    Ok((summary, rows))
}

fn parse_catalog() -> Result<UniversalCatalog, AwsScanError> {
    if UNIVERSAL_CATALOG_JSON.trim().is_empty() {
        return Err(invalid("embedded universal Cloud Control catalog is empty"));
    }
    let parsed: UniversalCatalog = serde_json::from_str(UNIVERSAL_CATALOG_JSON)
        .map_err(|error| invalid(format!("embedded universal Cloud Control catalog: {error}")))?;
    if parsed.schema_version != 1 {
        return Err(invalid(
            "unexpected universal Cloud Control catalog version",
        ));
    }
    Ok(parsed)
}

fn check_catalog_size(
    rows: &[UniversalRow],
    summary: &UniversalSummary,
) -> Result<(), AwsScanError> {
    if rows.is_empty() || rows.len() > MAX_UNIVERSAL_TYPES || rows.len() != summary.total {
        return Err(invalid(
            "unexpected universal Cloud Control catalog type count",
        ));
    }
    Ok(())
}

/// Per-capability row totals, used to prove the summary partitions the rows.
#[derive(Default)]
struct CapabilityCounts {
    list: usize,
    parent: usize,
    unsupported: usize,
}

fn count_capabilities(rows: &[UniversalRow]) -> Result<CapabilityCounts, AwsScanError> {
    let mut counts = CapabilityCounts::default();
    let mut names = BTreeSet::new();
    for row in rows {
        validate_row(row, &mut names, &mut counts)?;
    }
    Ok(counts)
}

fn validate_row<'a>(
    row: &'a UniversalRow,
    names: &mut BTreeSet<&'a str>,
    counts: &mut CapabilityCounts,
) -> Result<(), AwsScanError> {
    validate_identifier(&row.type_name, "Cloud Control type")?;
    if !row.type_name.contains("::") || !names.insert(row.type_name.as_str()) {
        return Err(invalid("invalid or duplicate universal Cloud Control type"));
    }
    let counter = capability_counter(row, counts)?;
    *counter = counter
        .checked_add(1)
        .ok_or_else(|| invalid("universal type count overflow"))?;
    Ok(())
}

fn capability_counter<'a>(
    row: &UniversalRow,
    counts: &'a mut CapabilityCounts,
) -> Result<&'a mut usize, AwsScanError> {
    let counter = match row.capability.as_str() {
        "list" => {
            validate_digest(&row.schema_sha256)?;
            &mut counts.list
        }
        "parent" => {
            if row.parent_type.is_none() && row.exclusion.is_none() {
                return Err(invalid("parent-dependent type lacks its reason"));
            }
            &mut counts.parent
        }
        "unsupported" => &mut counts.unsupported,
        _ => return Err(invalid("unknown universal capability classification")),
    };
    Ok(counter)
}

fn check_summary_partition(
    counts: &CapabilityCounts,
    summary: &UniversalSummary,
    rows: usize,
) -> Result<(), AwsScanError> {
    let classified = (counts.list, counts.parent, counts.unsupported)
        == (
            summary.list_capable,
            summary.parent_required,
            summary.unsupported,
        );
    let total = counts
        .list
        .checked_add(counts.parent)
        .and_then(|count| count.checked_add(counts.unsupported));
    if !classified || total != Some(rows) {
        return Err(invalid(
            "universal catalog summary does not partition its rows",
        ));
    }
    Ok(())
}
fn validate_digest(digest: &str) -> Result<(), AwsScanError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("invalid universal schema digest"));
    }
    Ok(())
}

/// Digest of the exact embedded bytes; the report never reads the filesystem.
pub fn digest() -> String {
    format!(
        "{:x}",
        base16ct::HexDisplay(&Sha256::digest(UNIVERSAL_CATALOG_JSON.as_bytes()))
    )
}

/// Complete catalog accounting: every row is either list-capable or an explicit
/// skip category.
pub fn report() -> Result<UniversalReport, AwsScanError> {
    let (summary, rows) = catalog()?;
    let mut skipped_parent_required = Vec::new();
    let mut skipped_unsupported = Vec::new();
    for row in &rows {
        match row.capability.as_str() {
            "parent" => skipped_parent_required.push(row.type_name.clone()),
            "unsupported" => skipped_unsupported.push(row.type_name.clone()),
            _ => {}
        }
    }
    Ok(UniversalReport {
        catalog_digest: digest(),
        total: summary.total,
        list_capable: summary.list_capable,
        parent_required: summary.parent_required,
        unsupported: summary.unsupported,
        skipped_parent_required,
        skipped_unsupported,
    })
}

/// Descriptors for every list-capable catalog row, in catalog order. Reviewed
/// descriptors are reused so identity, names and columns stay consistent;
/// unreviewed rows get an honest list-only descriptor derived from the row.
pub fn list_descriptors() -> Result<Vec<ResourceType>, AwsScanError> {
    let (_, rows) = catalog()?;
    let reviewed: BTreeMap<String, ResourceType> = crate::cloud_control::catalog()?
        .into_iter()
        .filter(|descriptor| !descriptor.list_requires_model)
        .map(|descriptor| (descriptor.type_name.clone(), descriptor))
        .collect();
    let mut kinds: BTreeSet<String> = reviewed.values().map(|row| row.kind.clone()).collect();
    let mut views: BTreeSet<String> = reviewed.values().map(|row| row.view_name.clone()).collect();
    let sensitive = crate::cloud_control::global_sensitive_names()?;
    let mut descriptors = Vec::new();
    for row in rows.iter().filter(|row| row.capability == "list") {
        if let Some(descriptor) = reviewed.get(&row.type_name) {
            descriptors.push(descriptor.clone());
        } else {
            descriptors.push(synthesized(row, &mut kinds, &mut views, &sensitive)?);
        }
    }
    Ok(descriptors)
}

fn synthesized(
    row: &UniversalRow,
    kinds: &mut BTreeSet<String>,
    views: &mut BTreeSet<String>,
    sensitive: &Arc<BTreeSet<String>>,
) -> Result<ResourceType, AwsScanError> {
    validate_digest(&row.schema_sha256)?;
    let names = SyntheticNames::resolve(row, kinds, views)?;
    Ok(names.descriptor(row, sensitive))
}

/// Deterministic names for one synthesized descriptor, resolved before the
/// descriptor is built so descriptor ordering stays catalog order.
struct SyntheticNames {
    kind: String,
    view_name: String,
    service: String,
}

impl SyntheticNames {
    fn resolve(
        row: &UniversalRow,
        kinds: &mut BTreeSet<String>,
        views: &mut BTreeSet<String>,
    ) -> Result<Self, AwsScanError> {
        let normalized = normalized_name(&row.type_name)?;
        let suffix = short_digest(&row.type_name);
        Ok(Self {
            kind: unique_name("aws_cc_", &normalized, &suffix, kinds)?,
            view_name: unique_name(UNIVERSAL_VIEW_PREFIX, &normalized, &suffix, views)?,
            service: normalized.split('_').next().unwrap_or("unknown").to_owned(),
        })
    }

    fn descriptor(&self, row: &UniversalRow, sensitive: &Arc<BTreeSet<String>>) -> ResourceType {
        ResourceType {
            type_name: row.type_name.clone(),
            kind: self.kind.clone(),
            view_name: self.view_name.clone(),
            service: self.service.clone(),
            scope: "regional".to_owned(),
            list_supported: true,
            list_model_validation: row
                .exclusion
                .clone()
                .unwrap_or_else(|| "universal_capability_matrix".to_owned()),
            primary_identifier: vec!["/Identifier".to_owned()],
            schema_sha256: row.schema_sha256.clone(),
            sensitive_names: Arc::clone(sensitive),
            ..ResourceType::default()
        }
    }
}

//! Universal-catalog evidence and list-only request accounting.
use super::*;

const UNIVERSAL_CATALOG: &str = include_str!("../../catalog/cloudcontrol-universal.json");

/// The universal catalog is data, not a collector dependency: planning can
/// report its partition without the `aws` feature. Per-type plans remain the
/// reviewed subset that carries executable descriptors.
pub(super) fn evidence() -> Result<(u64, Value)> {
    let document: Value =
        serde_json::from_str(UNIVERSAL_CATALOG).context("embedded universal catalog")?;
    let summary = document
        .get("summary")
        .context("universal catalog summary is missing")?;
    let list_capable = check_summary(&document, summary)?;
    Ok((list_capable, evidence_document(summary)))
}

fn check_summary(document: &Value, summary: &Value) -> Result<u64> {
    let (total, list_capable, parent_required, unsupported) = counts(summary)?;
    let rows = document
        .get("resource_types")
        .and_then(Value::as_array)
        .context("universal catalog rows are missing")?;
    ensure!(
        partitions(
            rows.len(),
            total,
            list_capable,
            parent_required,
            unsupported
        ),
        "universal catalog rows do not partition their summary"
    );
    Ok(list_capable)
}

fn evidence_document(summary: &Value) -> Value {
    json!({
        "catalog":"catalog/cloudcontrol-universal.json","summary":summary,
        "note":"Every list-capable row is attempted with ListResources; parent and unsupported rows are explicit skips. Only reviewed rows have executable per-type plans.",
    })
}

fn counts(summary: &Value) -> Result<(u64, u64, u64, u64)> {
    let total = count(summary, "total", "universal catalog total is missing")?;
    let list_capable = count(
        summary,
        "list_capable",
        "universal catalog list count is missing",
    )?;
    let parent_required = count(
        summary,
        "parent_required",
        "universal catalog parent count is missing",
    )?;
    let unsupported = count(
        summary,
        "unsupported",
        "universal catalog unsupported count is missing",
    )?;
    Ok((total, list_capable, parent_required, unsupported))
}

fn count(summary: &Value, key: &str, message: &'static str) -> Result<u64> {
    summary.get(key).and_then(Value::as_u64).context(message)
}

fn partitions(rows: usize, total: u64, list_capable: u64, parent: u64, unsupported: u64) -> bool {
    u64::try_from(rows).ok() == Some(total)
        && list_capable
            .checked_add(parent)
            .and_then(|count| count.checked_add(unsupported))
            == Some(total)
}

/// Mark the plan as a universal drain and expose list-only request accounting.
pub(super) fn annotate(plan: &mut Value, list_types: u64, evidence: Value) -> Result<()> {
    let coverage = plan
        .get_mut("coverage")
        .and_then(Value::as_object_mut)
        .context("plan coverage must be an object")?;
    coverage.insert("resource_type_all".into(), Value::Bool(true));
    coverage.insert("universal".into(), evidence);
    let accounting = plan
        .get_mut("request_accounting")
        .and_then(Value::as_object_mut)
        .context("plan request accounting must be an object")?;
    accounting.insert("universal_mode".into(), Value::Bool(true));
    accounting.insert("universal_list_types".into(), json!(list_types));
    accounting.insert("universal_get_resource_requests".into(), json!(0));
    accounting.insert("universal_parent_collection_requests".into(), json!(0));
    accounting.insert(
        "universal_note".into(),
        json!("Universal mode performs one ListResources cursor per list-capable type and region; parent workflows, GetResource enrichment and child scopes are not run."),
    );
    Ok(())
}

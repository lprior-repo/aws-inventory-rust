use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const CAPABILITIES: &str = include_str!("../catalog/cloudcontrol-capabilities.toml");

pub fn report(
    service: Option<&str>,
    type_name: Option<&str>,
    unselected: bool,
    limit: usize,
) -> Result<Value> {
    ensure!(
        (1..=4_096).contains(&limit),
        "coverage limit must be 1..4096"
    );
    for filter in [service, type_name].into_iter().flatten() {
        ensure!(
            !filter.is_empty() && filter.len() <= 256 && !filter.contains('\0'),
            "invalid coverage filter"
        );
    }
    let mut catalog: Value = toml::from_str(CAPABILITIES)?;
    merge_parent_capabilities(&mut catalog)?;
    let types = catalog
        .get("resource_types")
        .and_then(Value::as_array)
        .context("capability types missing")?;
    ensure!(
        !types.is_empty() && types.len() <= 4_096,
        "capability catalog exceeds schema bound"
    );
    let matched = |entry: &&Value| {
        let family = |field: &str, wanted: &str| {
            entry
                .get(field)
                .and_then(Value::as_str)
                .is_some_and(|actual| actual.eq_ignore_ascii_case(wanted))
        };
        service.is_none_or(|wanted| family("service", wanted) || family("service_name", wanted))
            && type_name
                .is_none_or(|wanted| entry.get("type_name").and_then(Value::as_str) == Some(wanted))
            && (!unselected || entry.get("selected") == Some(&json!(false)))
    };
    let matched_count = types.iter().filter(matched).count();
    let rows: Vec<&Value> = types.iter().filter(matched).take(limit).collect();
    Ok(
        json!({"schema_version":1,"status":"complete","mode":"offline_capability_evidence",
        "summary":catalog.get("summary"),"evidence":catalog.get("evidence"),
        "matched":matched_count,"returned":rows.len(),"limit":limit,"truncated":matched_count > rows.len(),
        "resource_types":rows,
        "semantics":"selected means an executable reviewed collector descriptor; emulator outcomes are separately recorded, and neither proves live account/region availability"}),
    )
}

fn merge_parent_capabilities(catalog: &mut Value) -> Result<()> {
    let parents: Value = toml::from_str(crate::plan::PARENT_CATALOG)?;
    let types = parents
        .get("resource_types")
        .and_then(Value::as_array)
        .context("missing parent resource descriptors")?;
    let plans = parents
        .get("parent_plans")
        .and_then(Value::as_array)
        .context("missing parent workflows")?;
    ensure!(
        types.len() == 24 && plans.len() == 24,
        "invalid parent capability bound"
    );
    let rows = catalog
        .get_mut("resource_types")
        .and_then(Value::as_array_mut)
        .context("missing capability types")?;
    ensure!(rows.len() <= 4_096, "capability bound exceeded");
    let mut added = 0usize;
    for descriptor in types {
        let name = descriptor
            .get("type_name")
            .and_then(Value::as_str)
            .context("missing parent type name")?;
        let row = rows
            .iter_mut()
            .find(|row| row.get("type_name").and_then(Value::as_str) == Some(name))
            .context("parent type absent from pinned schema universe")?;
        ensure!(
            row.get("selected") == Some(&json!(false))
                && row.get("schema_sha256") == descriptor.get("schema_sha256"),
            "parent type overlaps or differs from the schema universe"
        );
        let plan = plans
            .iter()
            .find(|plan| plan.get("type_name").and_then(Value::as_str) == Some(name))
            .context("parent workflow missing")?;
        let relationships = descriptor
            .get("relationships")
            .and_then(Value::as_array)
            .context("parent relationships missing")?;
        let row = row.as_object_mut().context("invalid capability row")?;
        row.insert("selected".into(), json!(true));
        row.insert(
            "prior_exclusion".into(),
            row.get("exclusion").cloned().unwrap_or(Value::Null),
        );
        row.insert("exclusion".into(), Value::Null);
        row.insert("selection_kind".into(), json!("parent_dependent"));
        row.insert(
            "scope_review".into(),
            json!("regional_with_reviewed_parent"),
        );
        row.insert(
            "list_model_validation".into(),
            json!("reviewed_parent_model"),
        );
        row.insert("parent_workflow".into(), plan.clone());
        row.insert(
            "view_name".into(),
            descriptor
                .get("view_name")
                .cloned()
                .context("parent view missing")?,
        );
        row.insert("relationship_rule_count".into(), json!(relationships.len()));
        row.insert("parent_relationship_rule_count".into(), json!(1));
        added = added
            .checked_add(1)
            .context("parent capability count overflow")?;
    }
    ensure!(added == 24, "parent capability set incomplete");
    let mut exclusions = BTreeMap::<String, usize>::new();
    for row in rows
        .iter()
        .filter(|row| row.get("selected") == Some(&json!(false)))
    {
        let reason = row
            .get("exclusion")
            .and_then(Value::as_str)
            .context("exclusion missing")?;
        let count = exclusions.entry(reason.to_owned()).or_default();
        *count = count.checked_add(1).context("exclusion count overflow")?;
    }
    let selected = rows
        .iter()
        .filter(|row| row.get("selected") == Some(&json!(true)))
        .count();
    let relationship_count = rows
        .iter()
        .filter_map(|row| row.get("relationship_rule_count").and_then(Value::as_u64))
        .try_fold(0u64, |total, count| total.checked_add(count))
        .context("relationship count overflow")?;
    let linked_types = rows
        .iter()
        .filter(|row| {
            row.get("selected") == Some(&json!(true))
                && (row
                    .get("relationship_rule_count")
                    .and_then(Value::as_u64)
                    .is_some_and(|count| count > 0)
                    || row.get("parent_workflow").is_some())
        })
        .count();
    let excluded = rows
        .len()
        .checked_sub(selected)
        .context("invalid selection count")?;
    let summary = catalog
        .get_mut("summary")
        .and_then(Value::as_object_mut)
        .context("summary missing")?;
    summary.insert("selected_count".into(), json!(selected));
    summary.insert("excluded_count".into(), json!(excluded));
    summary.insert("exclusions".into(), json!(exclusions));
    summary.insert("root_only_selected_count".into(), json!(397));
    summary.insert("parent_dependent_selected_count".into(), json!(added));
    summary.insert("relationship_rule_count".into(), json!(relationship_count));
    summary.insert("parent_relationship_rule_count".into(), json!(added));
    summary.insert("types_with_relationships".into(), json!(linked_types));
    summary.insert("emulator_evidence_basis".into(), json!("Retained v0.5 root-catalog receipts remain source-bound; parent workflow execution evidence is reported separately."));
    catalog
        .get_mut("evidence")
        .and_then(Value::as_object_mut)
        .context("evidence missing")?
        .insert(
            "parent_workflows".into(),
            parents
                .get("evidence")
                .cloned()
                .context("parent evidence missing")?,
        );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_matrix_and_selected_execution_scope_are_distinct() -> Result<()> {
        let full = report(None, None, false, 4_096)?;
        assert_eq!(full.get("matched"), Some(&json!(1_881)));
        assert_eq!(full.get("truncated"), Some(&json!(false)));
        let rows = full
            .get("resource_types")
            .and_then(Value::as_array)
            .context("rows")?;
        assert_eq!(
            rows.iter()
                .filter(|r| r.get("selected") == Some(&json!(true)))
                .count(),
            421
        );
        let excluded = report(None, None, true, 4_096)?;
        assert_eq!(excluded.get("matched"), Some(&json!(1_460)));
        assert_eq!(full.pointer("/summary/selected_count"), Some(&json!(421)));
        assert_eq!(full.pointer("/summary/excluded_count"), Some(&json!(1460)));
        assert_eq!(
            full.pointer("/summary/root_only_selected_count"),
            Some(&json!(397))
        );
        assert_eq!(
            full.pointer("/summary/relationship_rule_count"),
            Some(&json!(968))
        );
        assert_eq!(
            full.pointer("/summary/parent_relationship_rule_count"),
            Some(&json!(24))
        );
        assert_eq!(
            full.pointer("/summary/types_with_relationships"),
            Some(&json!(268))
        );
        assert_eq!(
            full.pointer("/summary/exclusions"),
            Some(&json!({
                "missing_read_or_list_handler":180,"scope_unreviewed":894,
                "schema_parent_model_required":354,"documented_parent_model_required":32
            }))
        );
        Ok(())
    }

    #[test]
    fn coverage_filtering_retains_exclusion_and_truncation_evidence() -> Result<()> {
        let first = report(Some("EC2"), None, false, 1)?;
        assert_eq!(first.get("returned"), Some(&json!(1)));
        assert_eq!(first.get("truncated"), Some(&json!(true)));
        let specific = report(None, Some("AWS::EC2::Instance"), false, 1)?;
        assert_eq!(specific.get("matched"), Some(&json!(1)));
        assert_eq!(
            specific.pointer("/resource_types/0/selected"),
            Some(&json!(true))
        );
        assert_eq!(
            report(Some("no-such-service"), None, false, 1)?.get("matched"),
            Some(&json!(0))
        );
        assert!(report(None, None, false, 0).is_err());
        assert!(report(None, None, false, 4_097).is_err());
        Ok(())
    }

    #[test]
    fn parent_capability_retains_source_and_verification_boundaries() -> Result<()> {
        let result = report(None, Some("AWS::EKS::Addon"), false, 1)?;
        assert_eq!(
            result.pointer("/resource_types/0/selected"),
            Some(&json!(true))
        );
        assert_eq!(
            result.pointer("/resource_types/0/prior_exclusion"),
            Some(&json!("documented_parent_model_required"))
        );
        assert_eq!(
            result.pointer("/resource_types/0/parent_workflow/parent_type"),
            Some(&json!("AWS::EKS::Cluster"))
        );
        assert_eq!(
            result.pointer("/resource_types/0/emulator_verification/status"),
            Some(&json!("not_run"))
        );
        assert_eq!(
            result.pointer("/summary/parent_dependent_selected_count"),
            Some(&json!(24))
        );
        assert_eq!(
            report(None, Some("AWS::EKS::Addon"), true, 1)?.get("matched"),
            Some(&json!(0))
        );
        Ok(())
    }
}

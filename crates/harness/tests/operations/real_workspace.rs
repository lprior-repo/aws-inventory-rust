//! Real-workspace operation partition, including the reviewed-addition trigger.
use super::*;

#[test]
fn real_workspace_inventory_is_fully_partitioned() -> TestResult {
    // Given: the current source-only inventory, reviewed additions and frozen outcomes.
    // When: coverage joins them to the active lockfile versions.
    let report = operation_coverage(&workspace())?.to_value()?;
    let total = number(&report, "/summary/operations_total")?;
    let exercised = number(&report, "/summary/exercised")?;
    let unexercised = number(&report, "/summary/unexercised")?;
    // Then: no upgraded SDK inherits historical execution credit.
    assert_eq!(total, 24_720);
    assert_eq!(number(&report, "/summary/crates_total")?, 468);
    assert_eq!(exercised, 47);
    assert_eq!(exercised.checked_add(unexercised), Some(total));
    let services = array(&report, "/by_service")?;
    assert_eq!(service_sum(services, "total")?, total);
    assert_eq!(service_sum(services, "exercised")?, exercised);
    additions_stay_unexercised(&report)?;
    Ok(())
}

fn service_sum(rows: &[Value], field: &str) -> Result<u64, BoxError> {
    rows.iter().try_fold(0_u64, |total, row| {
        let count = row
            .get(field)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("missing service {field}"))?;
        total
            .checked_add(count)
            .ok_or_else(|| BoxError::from("service total overflow"))
    })
}

fn additions_stay_unexercised(report: &Value) -> TestResult {
    assert_eq!(number(report, "/summary/reviewed_additions_exercised")?, 0);
    assert!(
        array(report, "/exercised_operations")?
            .iter()
            .all(|row| row.get("service").and_then(Value::as_str) != Some("kms"))
    );
    Ok(())
}

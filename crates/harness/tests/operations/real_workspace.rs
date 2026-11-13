//! Real-workspace operation partition, including the reviewed-addition trigger.
use super::*;

#[test]
fn real_workspace_inventory_is_fully_partitioned() -> TestResult {
    let report = match operation_coverage(&workspace()) {
        Ok(report) => report.to_value()?,
        Err(HarnessError::Invalid(reason))
            if reason.contains("add reviewed rows to catalog/aws-operation-additions.json") =>
        {
            assert!(reason.contains("aws-sdk-kms"), "{reason}");
            return Ok(());
        }
        Err(other) => return Err(other.into()),
    };
    let total = number(&report, "/summary/operations_total")?;
    let exercised = number(&report, "/summary/exercised")?;
    let unexercised = number(&report, "/summary/unexercised")?;
    assert_eq!(total, 20_537);
    assert_eq!(number(&report, "/summary/crates_total")?, 440);
    assert_eq!(exercised.checked_add(unexercised), Some(total));
    assert!(exercised > 0);
    let services = array(&report, "/by_service")?;
    let service_total: u64 = services
        .iter()
        .map(|row| row.get("total").and_then(Value::as_u64).unwrap_or(0))
        .sum();
    let service_exercised: u64 = services
        .iter()
        .map(|row| row.get("exercised").and_then(Value::as_u64).unwrap_or(0))
        .sum();
    assert_eq!(service_total, total);
    assert_eq!(service_exercised, exercised);
    assert_eq!(number(&report, "/summary/reviewed_additions_exercised")?, 0);
    assert!(
        array(&report, "/exercised_operations")?
            .iter()
            .all(|row| { row.get("service").and_then(Value::as_str) != Some("kms") })
    );
    Ok(())
}

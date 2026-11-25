use super::{TestResult, runtime::workspace};
use inventory_core::catalog::runtime::{PlanPurpose, RuntimeCatalog};
use std::collections::BTreeSet;

pub async fn verify(report: &str) -> TestResult {
    let bytes = tokio::fs::read(workspace().join("catalog/aws-api.catalog")).await?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    verify_coverage(&catalog, report);
    verify_tag_children(&catalog, report);
    let protocols = item_protocols(report);
    assert!(
        protocols.len() >= 5,
        "fewer than five item-bearing protocols: {protocols:?}"
    );
    for line in report
        .lines()
        .filter(|line| line.starts_with("total ") || line.starts_with("protocol "))
    {
        println!("{line}");
    }
    Ok(())
}

fn verify_coverage(catalog: &RuntimeCatalog<'_>, report: &str) {
    let rows = rows(report);
    let expected = catalog
        .plans()
        .iter()
        .map(|plan| (plan.service_name(), plan.operation_name()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rows, expected,
        "every callable must have exactly one evidence row"
    );
    assert_eq!(
        catalog.plans().len(),
        expected.len(),
        "duplicate callable catalog keys"
    );
    let count = report
        .lines()
        .filter(|line| !line.starts_with("total ") && !line.starts_with("protocol "))
        .count();
    assert_eq!(count, rows.len(), "duplicate evidence rows");
}

fn verify_tag_children(catalog: &RuntimeCatalog<'_>, report: &str) {
    for plan in catalog
        .plans()
        .iter()
        .filter(|plan| matches!(plan.purpose(), PlanPurpose::TagEndpoint(_)))
    {
        let line = report.lines().find(|line| {
            let mut words = line.split_whitespace();
            words.next() == Some(plan.service_name()) && words.next() == Some(plan.operation_name())
        });
        assert!(
            line.and_then(|line| line.splitn(4, ' ').nth(3))
                == Some("skipped child tag endpoint uses dedicated resource tag binding"),
            "tag endpoint must report the dedicated binding skip: {} {}: {line:?}",
            plan.service_name(),
            plan.operation_name()
        );
    }
}

fn rows(report: &str) -> BTreeSet<(&str, &str)> {
    report
        .lines()
        .filter(|line| !line.starts_with("total ") && !line.starts_with("protocol "))
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            Some((words.next()?, words.next()?))
        })
        .collect()
}

fn item_protocols(report: &str) -> BTreeSet<&str> {
    report
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            words.next()?;
            words.next()?;
            let protocol = words.next()?;
            (words.next()? == "items").then_some(protocol)
        })
        .collect()
}

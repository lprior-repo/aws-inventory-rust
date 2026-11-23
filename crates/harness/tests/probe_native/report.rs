use super::{TestResult, runtime::workspace};
use inventory_core::catalog::runtime::{PlanPurpose, RuntimeCatalog};
use std::collections::BTreeSet;

const THIN: &[&str] = &[
    "ec2",
    "sts",
    "iam",
    "sns",
    "sqs",
    "dynamodb",
    "kms",
    "lambda",
    "s3",
    "cloudwatch",
];

pub async fn verify(report: &str) -> TestResult {
    let bytes = tokio::fs::read(workspace().join("catalog/aws-api.catalog")).await?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    verify_coverage(&catalog, report);
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
    let plans = catalog.plans().iter().filter(|plan| {
        matches!(plan.purpose(), PlanPurpose::Enumeration) && THIN.contains(&plan.service_name())
    });
    for plan in plans {
        assert!(
            rows.contains(&(plan.service_name(), plan.operation_name())),
            "missing thin enumeration {} {}",
            plan.service_name(),
            plan.operation_name()
        );
    }
}

fn rows(report: &str) -> BTreeSet<(&str, &str)> {
    report
        .lines()
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

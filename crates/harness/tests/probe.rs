#[path = "probe/cli.rs"]
mod cli;
#[path = "probe/context.rs"]
mod context;
#[path = "../../catalog-tool/tests/support/bindings.rs"]
mod fixture;
#[path = "probe/proof.rs"]
mod proof;
#[path = "probe/server.rs"]
mod server;

use inventory_core::catalog::runtime::RuntimeCatalog;
use server::{Server, TestResult};

const FIRST_ROWS: [&str; 3] = [
    "bindings ListChildren awsJson1_1 skipped child",
    "bindings ListParents0 awsJson1_1 items 2",
    "bindings ListParents1 awsJson1_1 empty",
];
const PROTOCOL_TOTAL: &str =
    "protocol awsJson1_1 items 1 empty 1 unsupported 1 failed 1 skipped-child 1";

#[tokio::test]
async fn given_four_outcomes_when_probed_twice_then_rows_and_totals_are_stable() -> TestResult {
    let bytes = fixture::compiled(4, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (report, requests) = proof::twice(&catalog).await?;
    assert_eq!(report.lines().take(3).collect::<Vec<_>>(), FIRST_ROWS);
    assert!(report.contains("ListParents2 awsJson1_1 unsupported "));
    assert!(report.contains("ListParents3 awsJson1_1 failed decode shape="));
    assert_eq!(report.lines().last(), Some(PROTOCOL_TOTAL));
    assert_eq!(requests.len(), 8);
    Ok(())
}

#[tokio::test]
async fn given_next_token_when_calling_once_then_only_first_page_and_http_version_return()
-> TestResult {
    let bytes = fixture::compiled(4, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (answer, requests) = proof::once(&catalog).await?;
    assert_eq!(answer.items, 2);
    assert_eq!(answer.http_version, http::Version::HTTP_11);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_child_plan_when_calling_once_then_root_contract_rejects_before_transport()
-> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let context = context::Context::new(&server)?;
    assert!(matches!(
        context.once(&catalog, "ListChildren").await,
        Err(inventory_sweep::SweepError::InvalidConfig(
            "probe requires a root enumeration from this catalog"
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_no_python_when_probe_cli_runs_then_report_contains_every_enumeration() -> TestResult
{
    let bytes = fixture::compiled(4, |_| {})?;
    let report = cli::report(&bytes).await?;
    assert_eq!(
        report
            .lines()
            .filter(|line| line.starts_with("bindings "))
            .count(),
        5
    );
    assert_eq!(
        report.lines().next(),
        Some("bindings ListChildren awsJson1_1 skipped child")
    );
    assert!(report.contains("bindings ListParents0 awsJson1_1 items 2"));
    Ok(())
}

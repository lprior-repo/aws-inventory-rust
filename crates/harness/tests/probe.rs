#[path = "probe/children.rs"]
mod children;
#[path = "probe/cli.rs"]
mod cli;
#[path = "probe/comparison.rs"]
mod comparison;
#[path = "probe/context.rs"]
mod context;
#[path = "../../catalog-tool/tests/support/bindings.rs"]
mod fixture;
#[path = "probe/pair.rs"]
mod pair;
#[path = "probe/proof.rs"]
mod proof;
#[path = "probe/rest.rs"]
mod rest;
#[path = "probe/scoped.rs"]
mod scoped;
#[path = "probe/server.rs"]
mod server;
#[path = "probe/skip_execution.rs"]
mod skip_execution;
#[path = "../../catalog-tool/tests/support/skip_runtime.rs"]
mod skip_runtime;

use inventory_core::catalog::runtime::RuntimeCatalog;
use server::{Server, TestResult};

const FIRST_ROWS: [&str; 3] = [
    "bindings ListChildren awsJson1_1 items 1",
    "bindings ListParents0 awsJson1_1 items 2",
    "bindings ListParents1 awsJson1_1 empty",
];
const PROTOCOL_TOTAL: &str =
    "protocol awsJson1_1 items 2 empty 1 unsupported 1 failed 1 skipped-child 0";

#[tokio::test]
async fn given_four_outcomes_when_probed_twice_then_rows_and_totals_are_stable() -> TestResult {
    let bytes = fixture::compiled(4, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (report, requests) = proof::twice(&catalog).await?;
    assert_eq!(report.lines().take(3).collect::<Vec<_>>(), FIRST_ROWS);
    children::assert_row(
        &report,
        "bindings ListParents2 awsJson1_1 unsupported API executor error: provider does not support this type: AWS service failure: NotSupported (UnsupportedOperation)",
    );
    children::assert_row(
        &report,
        "bindings ListParents3 awsJson1_1 failed decode shape=Output AWS wire response error: invalid response: JSON syntax",
    );
    assert_eq!(report.lines().last(), Some(PROTOCOL_TOTAL));
    assert_eq!(requests.len(), 10);
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
    assert_eq!(
        requests
            .iter()
            .map(|request| server::target(request))
            .collect::<TestResult<Vec<_>>>()?,
        ["ListParents0"]
    );
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
        context.once(&catalog, "ListChildren", None).await,
        Err(inventory_sweep::SweepError::InvalidConfig(
            "probe requires a root callable from this catalog"
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_no_python_when_probe_cli_runs_then_report_contains_every_enumeration() -> TestResult
{
    let bytes = fixture::compiled(4, |_| {})?;
    let (report, requests) = cli::run(&bytes, &[], server::response).await?;
    assert_eq!(requests.len(), 5);
    assert_eq!(
        report
            .lines()
            .filter(|line| line.starts_with("bindings "))
            .count(),
        5
    );
    assert_eq!(
        report.lines().next(),
        Some("bindings ListChildren awsJson1_1 items 1")
    );
    assert!(
        report
            .lines()
            .any(|line| line == "bindings ListParents0 awsJson1_1 items 2")
    );
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn given_report_symlink_to_future_receipt_when_comparing_then_neither_is_written()
-> TestResult {
    let bytes = skip_runtime::catalog(skip_runtime::Case::Ordinary)?;
    let work = tempfile::tempdir()?;
    let report = work.path().join("probe.report");
    let receipt = work.path().join("probe.report.skip.toml");
    std::os::unix::fs::symlink(&receipt, &report)?;
    tokio::fs::write(work.path().join("probe.catalog"), bytes).await?;
    let server = Server::new().await?;
    let mut command = cli::command(&server, work.path())?;
    command.args([
        "--compare-pair",
        "skipruntime/ListThings/DescribeThing",
        "--compared-at",
        "2026-11-24T00:00:00Z",
    ]);
    let bodies = [
        r#"{"Account":"123456789012"}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    ];
    let (completed, completion) = tokio::sync::oneshot::channel();
    let (output, _) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::try_join!(
            async {
                let output = command.output().await?;
                completed.send(()).map_err(|_| "comparison server closed")?;
                Ok::<_, Box<dyn std::error::Error>>(output)
            },
            server.serve_until(&bodies, completion)
        )
    })
    .await??;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!receipt.try_exists()?);
    assert_eq!(std::fs::read_link(&report)?, receipt);
    Ok(())
}

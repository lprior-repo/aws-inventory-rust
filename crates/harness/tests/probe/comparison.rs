use super::{Server, TestResult, cli, skip_runtime};
use std::time::Duration;

#[tokio::test]
async fn given_equal_seeded_pair_when_cli_compares_then_receipt_is_stored_with_overlay_entry()
-> TestResult {
    let bytes = skip_runtime::catalog(skip_runtime::Case::Ordinary)?;
    let work = tempfile::tempdir()?;
    tokio::fs::write(work.path().join("probe.catalog"), &bytes).await?;
    let server = Server::new().await?;
    let mut command = cli::command(&server, work.path())?;
    command.args([
        "--compare-pair",
        "skipruntime/ListThings/DescribeThing",
        "--compared-at",
        "2026-11-24T00:00:00Z",
    ]);
    let (completed, completion) = tokio::sync::oneshot::channel();
    let bodies = [
        r#"{"Account":"123456789012"}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    ];
    let (output, requests) = tokio::time::timeout(Duration::from_secs(10), async {
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
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(requests.len(), 3);
    assert_eq!(
        tokio::fs::read_to_string(work.path().join("probe.report")).await?,
        "skipruntime/ListThings/DescribeThing\tequal\t1\n"
    );
    let stored = tokio::fs::read_to_string(work.path().join("probe.report.skip.toml")).await?;
    assert_eq!(stored.as_bytes(), output.stdout);
    let catalog = inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&bytes)?;
    let fingerprint = catalog_tool::overlay::SkipEvidence::fingerprint(
        &catalog,
        "skipruntime",
        "ListThings",
        "DescribeThing",
    )?;
    assert_receipt(&stored, &fingerprint)?;
    Ok(())
}

fn assert_receipt(stored: &str, fingerprint: &str) -> TestResult {
    let parsed: toml::Value = toml::from_str(stored)?;
    let parent = parsed
        .get("service")
        .and_then(|value| value.get("skipruntime"))
        .and_then(|value| value.get("operation"))
        .and_then(|value| value.get("ListThings"))
        .ok_or("parent missing")?;
    let receipt = parent
        .get("skip_evidence")
        .and_then(|value| value.get("DescribeThing"))
        .ok_or("receipt missing")?;
    assert_eq!(
        receipt.get("inputs_sha256").and_then(toml::Value::as_str),
        Some(fingerprint)
    );
    assert_eq!(
        receipt.get("resources").and_then(toml::Value::as_integer),
        Some(1)
    );
    assert_eq!(
        receipt.get("when").and_then(toml::Value::as_str),
        Some("2026-11-24T00:00:00Z")
    );
    assert_eq!(
        receipt.get("compared").and_then(toml::Value::as_str),
        Some(
            "ListThings.Items == DescribeThing.Items; all modeled describe members and raw values"
        )
    );
    Ok(())
}

async fn run_comparison(work: &std::path::Path) -> TestResult<std::process::Output> {
    let bytes = skip_runtime::catalog(skip_runtime::Case::Ordinary)?;
    tokio::fs::write(work.join("probe.catalog"), bytes).await?;
    let server = Server::new().await?;
    let mut command = cli::command(&server, work)?;
    command.args([
        "--compare-pair",
        "skipruntime/ListThings/DescribeThing",
        "--compared-at",
        "2026-11-24T00:00:00Z",
    ]);
    let (completed, completion) = tokio::sync::oneshot::channel();
    let bodies = [
        r#"{"Account":"123456789012"}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    ];
    let (output, requests) = tokio::time::timeout(Duration::from_secs(10), async {
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
    assert_eq!(requests.len(), 3);
    Ok(output)
}

#[tokio::test]
async fn given_existing_receipt_when_cli_compares_then_it_is_refused_without_replacing_bytes()
-> TestResult {
    let work = tempfile::tempdir()?;
    let receipt = work.path().join("probe.report.skip.toml");
    tokio::fs::write(&receipt, b"operator-reviewed receipt\n").await?;
    let output = run_comparison(work.path()).await?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains(&receipt.display().to_string()));
    assert_eq!(
        tokio::fs::read(&receipt).await?,
        b"operator-reviewed receipt\n"
    );
    assert!(!work.path().join("probe.report").exists());
    assert!(output.stdout.is_empty());
    Ok(())
}

#[tokio::test]
async fn given_unwritable_receipt_destination_when_cli_compares_then_no_report_is_written()
-> TestResult {
    let work = tempfile::tempdir()?;
    let receipt = work.path().join("probe.report.skip.toml");
    tokio::fs::create_dir(&receipt).await?;
    let output = run_comparison(work.path()).await?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains(&receipt.display().to_string()));
    assert!(!work.path().join("probe.report").exists());
    assert!(receipt.is_dir());
    assert!(output.stdout.is_empty());
    Ok(())
}

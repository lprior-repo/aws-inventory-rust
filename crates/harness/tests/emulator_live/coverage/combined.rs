use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the pinned emulator binary and a collector build"]
async fn given_owned_emulator_when_native_and_universal_scan_runs_then_both_are_present()
-> TestResult {
    let work = tempfile::tempdir()?;
    let path = work.path().to_path_buf();
    let binary = emulator_binary()?;
    let emulator = tokio::task::spawn_blocking(move || Emulator::start(&binary, &path, 900))
        .await
        .map_err(|_| "emulator boot task panicked")??;
    let environment = emulator.environment();
    let seeded = run(
        Command::new(env!("CARGO_BIN_EXE_inventory-harness")).args([
            "seed",
            "s3",
            "--bucket",
            super::super::BUCKET,
        ]),
        &environment,
    )?;
    assert!(seeded.status.success(), "seeding failed");

    let database = work.path().join("combined.sqlite");
    let database_text = text(&database)?;
    let report = scan(
        &database_text,
        &[
            "--service",
            "all,cloud-control",
            "--cloud-control",
            "--resource-type",
            "all",
        ],
        &environment,
    )?;
    assert_terminal(&report, "combined");
    assert_eq!(
        report.get("status").and_then(Value::as_str),
        Some("complete")
    );
    let block = report
        .pointer("/scans/0/universal")
        .ok_or("combined scan lacks its universal partition")?;
    assert_eq!(number(block, "attempted")?, number(block, "list_capable")?);
    let resources = report
        .pointer("/scans/0/summary/resources")
        .and_then(Value::as_u64)
        .ok_or("combined scan lacks its resource count")?;
    assert!(resources > 0, "combined scan observed no resources");
    let options = SqliteConnectOptions::new()
        .filename(&database)
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let observed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM resource_observations WHERE resource_json LIKE ?")
            .bind(format!("%{}%", super::super::BUCKET))
            .fetch_one(&mut connection)
            .await?;
    connection.close().await?;
    assert!(
        observed > 0,
        "combined scan did not observe the seeded bucket"
    );
    Ok(())
}

use super::*;

#[test]
#[ignore = "requires the explicitly prepared local AWS Emulator fixture"]
fn emulator_cli_durable_worker_persists_an_assumed_role_scan() -> Result<()> {
    let snapshot = fixture()?;
    let expected = case(&snapshot, "assumedrole")?;
    let directory = tempfile::tempdir()?;
    let database = text_path(&directory.path().join("inventory.sqlite"))?;
    let queue = text_path(&directory.path().join("jobs.sqlite"))?;
    let role = snapshot
        .get("assume_role_arn")
        .and_then(Value::as_str)
        .context("assume-role fixture missing")?;
    let submitted = cli(
        &[
            "job",
            "submit",
            "--queue",
            &queue,
            "--job-id",
            "emulator-queued",
            "--db",
            &database,
            "--region",
            "us-east-1",
            "--service",
            "iam",
            "--role-arn",
            role,
            "--attempts",
            "2",
            "--max-pages",
            "100",
            "--page-size",
            "1",
            "--concurrency",
            "32",
            "--per-bucket-concurrency",
            "4",
            "--rps",
            "100",
        ],
        0,
    )?;
    assert_eq!(submitted.get("status"), Some(&json!("queued")));
    let before = now()?;
    let completed = cli(
        &[
            "job",
            "run",
            "--queue",
            &queue,
            "--limit",
            "1",
            "--lease-seconds",
            "60",
            "--worker",
            "emulator-worker",
        ],
        0,
    )?;
    let after = now()?;
    assert_eq!(completed.get("status"), Some(&json!("complete")));
    assert_eq!(completed.get("executed"), Some(&json!(1)));
    assert_eq!(completed.get("stop_reason"), Some(&json!("claim_limit")));
    let record = cli(
        &[
            "job",
            "show",
            "--queue",
            &queue,
            "--job-id",
            "emulator-queued",
        ],
        0,
    )?;
    assert_eq!(record.pointer("/job/state"), Some(&json!("succeeded")));
    assert_eq!(record.pointer("/job/attempt"), Some(&json!(1)));
    assert_eq!(
        record
            .get("attempts")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );
    let scans = query(&database, "SELECT scan_id,status FROM scans")?;
    assert_eq!(scans.len(), 1);
    let scan = scans.first().context("worker scan missing")?;
    assert_eq!(scan.get("status"), expected.get("status"));
    let id = scan
        .get("scan_id")
        .and_then(Value::as_str)
        .context("worker scan ID missing")?;
    ensure!(
        id.len() == 36
            && id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() || byte == b'-'),
        "invalid worker scan ID"
    );
    observations(&database, id, expected, before, after)?;
    let second = cli(
        &[
            "job",
            "run",
            "--queue",
            &queue,
            "--limit",
            "1",
            "--worker",
            "emulator-worker",
        ],
        0,
    )?;
    assert_eq!(second.get("status"), Some(&json!("complete")));
    assert_eq!(second.get("executed"), Some(&json!(0)));
    assert_eq!(second.get("stop_reason"), Some(&json!("queue_empty")));
    assert_eq!(query(&database, "SELECT scan_id FROM scans")?.len(), 1);
    Ok(())
}

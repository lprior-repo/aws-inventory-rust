use super::*;

#[test]
fn durable_cli_routes_preserve_idempotency_cancellation_and_explicit_retry() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let queue = path(&directory.path().join("jobs.sqlite"))?;
    let database_path = directory.path().join("inventory.sqlite");
    let database = path(&database_path)?;
    let submit = arguments(&[
        "job",
        "submit",
        "--queue",
        &queue,
        "--job-id",
        "property-job",
        "--db",
        &database,
        "--region",
        "us-east-1",
        "--service",
        "s3",
    ]);
    let first = json_command(&submit, 0)?;
    let repeated = json_command(&submit, 0)?;
    assert_eq!(first.get("status"), Some(&json!("queued")));
    assert_eq!(repeated.get("status"), Some(&json!("queued")));
    assert!(!database_path.exists());
    let jobs = json_command(&arguments(&["job", "list", "--queue", &queue]), 0)?;
    assert_eq!(
        jobs.get("jobs").and_then(Value::as_array).map(Vec::len),
        Some(1)
    );
    json_command(
        &arguments(&[
            "job",
            "cancel",
            "--queue",
            &queue,
            "--job-id",
            "property-job",
        ]),
        0,
    )?;
    let cancelled = json_command(
        &arguments(&["job", "show", "--queue", &queue, "--job-id", "property-job"]),
        0,
    )?;
    assert_eq!(cancelled.pointer("/job/state"), Some(&json!("cancelled")));
    json_command(
        &arguments(&[
            "job",
            "retry",
            "--queue",
            &queue,
            "--job-id",
            "property-job",
        ]),
        0,
    )?;
    let retried = json_command(
        &arguments(&["job", "show", "--queue", &queue, "--job-id", "property-job"]),
        0,
    )?;
    assert_eq!(retried.pointer("/job/state"), Some(&json!("queued")));
    assert_eq!(retried.pointer("/job/attempt"), Some(&json!(0)));
    assert!(!database_path.exists());
    Ok(())
}

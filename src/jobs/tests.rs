use super::*;
use crate::cli::{Cli, Command};
use clap::Parser;

#[tokio::test]
async fn queue_submission_is_offline_and_persists_paths_without_external_id_value() -> Result<()> {
    let folder = std::env::temp_dir().join(format!("inventory-job-cli-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&folder).await?;
    let queue = folder.join("jobs.sqlite");
    let db = folder.join("inventory.sqlite");
    let cli = Cli::try_parse_from([
        "aws-inventory",
        "job",
        "submit",
        "--queue",
        queue.to_str().context("queue path")?,
        "--db",
        db.to_str().context("db path")?,
        "--job-id",
        "offline-test",
        "--service",
        "ec2",
        "--role-arn",
        "arn:aws:iam::123456789012:role/reader",
        "--external-id-env",
        "UNSET_SYNTHETIC_EXTERNAL_ID",
    ])?;
    let Command::Job { command } = cli.command else {
        anyhow::bail!("expected job command");
    };
    let submitted = execute(command).await?;
    assert_eq!(
        submitted.pointer("/job/state").and_then(Value::as_str),
        Some("queued")
    );
    assert!(!db.exists());
    let store = JobStore::open(&queue).await?;
    let job = store.get("offline-test").await?;
    assert_eq!(
        job.payload
            .as_ref()
            .and_then(|v| v.pointer("/accounts/external_id_env"))
            .and_then(Value::as_str),
        Some("UNSET_SYNTHETIC_EXTERNAL_ID")
    );
    assert_eq!(job.attempt, 0);
    store.close().await;
    tokio::fs::remove_dir_all(folder).await?;
    Ok(())
}

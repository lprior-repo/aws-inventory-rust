use super::{
    SEEDED, TOPIC,
    fixture::{Fixture, KIND, execute, topic_report},
    server::{Server, TestResult},
    store,
};
use std::path::Path;

#[tokio::test]
async fn given_other_account_only_in_another_region_when_account_omitted_then_cli_succeeds()
-> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let foreign = store::resource("111111111111", "eu-west-1", KIND, "foreign")?;
    store::append(&fixture.database, &[foreign]).await?;
    let (output, requests) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_foreign_global_role_when_account_selected_then_cli_excludes_foreign_identity()
-> TestResult {
    let kind = "AWS::IAM::Role";
    let mut fixture = Fixture::new(
        &[],
        "unsupported",
        "resource\tAWS::IAM::Role\t123456789012\tglobal\tworker\n",
    )
    .await?;
    fixture.account = Some("123456789012".to_owned());
    let rows = [
        store::resource("123456789012", "global", kind, "worker")?,
        store::resource("111111111111", "global", kind, "foreign-global")?,
    ];
    store::append(&fixture.database, &rows).await?;
    listable(&fixture, kind)?;
    let (output, requests) = execute(
        &fixture,
        &[(200, r#"{"ResourceDescriptions":[{"Identifier":"worker"}]}"#)],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    let text = std::str::from_utf8(&output.stdout)?;
    let rows: Vec<_> = text
        .lines()
        .filter(|line| line.split_once('\t').is_some_and(|(name, _)| name == kind))
        .collect();
    assert_eq!(
        rows,
        ["AWS::IAM::Role\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0"]
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    let request = requests.first().ok_or("missing IAM request")?;
    assert!(request.contains(r#""TypeName":"AWS::IAM::Role""#));
    Ok(())
}

fn listable(fixture: &Fixture, kind: &str) -> TestResult {
    let path = fixture.work.path().join("universal.report");
    let mut report: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let row = report
        .get_mut("resource_types")
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|rows| {
            rows.iter_mut()
                .find(|row| row.get("type_name").and_then(serde_json::Value::as_str) == Some(kind))
        })
        .ok_or("missing mapped kind")?;
    *row.get_mut("capability").ok_or("missing capability")? = "list".into();
    std::fs::write(path, serde_json::to_vec(&report)?)?;
    Ok(())
}

async fn default_output(
    root: Option<&Path>,
    oracle: Option<&Path>,
    database: &str,
    replies: &[(u16, &str)],
) -> TestResult<(std::process::Output, Option<Vec<u8>>)> {
    let cwd = tempfile::tempdir()?;
    let server = Server::new().await?;
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command
        .current_dir(cwd.path())
        .args(["verify", "cloud-control", "--database"])
        .arg(database)
        .arg("--endpoint")
        .arg(server.endpoint()?)
        .arg("--ca-bundle")
        .arg(&server.ca)
        .args(["--region", "us-east-1", "--report"])
        .arg(cwd.path().join("default.report"))
        .env_clear()
        .env("AWS_ACCESS_KEY_ID", "TEST")
        .env("AWS_SECRET_ACCESS_KEY", "TEST")
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .kill_on_drop(true);
    if let Some(root) = root {
        command.arg("--root").arg(root);
    }
    if let Some(oracle) = oracle {
        command.arg("--oracle").arg(oracle);
    }
    let (done, finished) = tokio::sync::oneshot::channel();
    let output = async {
        let output = command.output().await?;
        let _ = done.send(());
        Ok::<_, std::io::Error>(output)
    };
    let (output, requests) = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        tokio::try_join!(output, server.serve(replies, finished))
    })
    .await??;
    assert_eq!(requests.len(), replies.len());
    let path = cwd.path().join("default.report");
    let report = path
        .try_exists()?
        .then(|| std::fs::read(path))
        .transpose()?;
    Ok((output, report))
}

#[tokio::test]
async fn given_default_inputs_under_root_when_run_from_another_cwd_then_comparison_succeeds()
-> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let root = fixture.work.path();
    let catalog = root.join("catalog");
    let seeds = root.join("crates/harness/seeds");
    std::fs::create_dir(&catalog)?;
    std::fs::create_dir_all(&seeds)?;
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
        catalog.join("aws-api.catalog"),
    )?;
    std::fs::rename(
        root.join("universal.report"),
        catalog.join("cloudcontrol-universal.json"),
    )?;
    std::fs::rename(
        &fixture.differences,
        catalog.join("cloudcontrol-differences.tsv"),
    )?;
    std::fs::write(
        seeds.join("sns.toml"),
        "format = 1\nservice = \"sns\"\n[[step]]\nid = \"topic\"\noperation = \"CreateTopic\"\nkind = \"AWS::SNS::Topic\"\nidentity = \"TopicArn\"\n[step.input]\nName = \"topic\"\n",
    )?;
    let database = format!("sqlite://{}", fixture.database.display());
    let (output, report) = default_output(Some(root), None, &database, &[(200, TOPIC)]).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(report.as_deref(), Some(output.stdout.as_slice()));
    Ok(())
}

#[tokio::test]
async fn given_default_differences_when_run_from_another_cwd_then_database_validation_is_reached()
-> TestResult {
    let oracle = tempfile::NamedTempFile::new()?;
    std::fs::write(oracle.path(), SEEDED)?;
    let (output, report) =
        default_output(None, Some(oracle.path()), "invalid://inventory", &[]).await?;
    assert_eq!(report, None);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control database must be a PostgreSQL or SQLite URL\n"
    );
    Ok(())
}

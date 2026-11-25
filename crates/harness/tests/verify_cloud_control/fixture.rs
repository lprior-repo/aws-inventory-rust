use super::{
    server::{Server, TestResult},
    store,
};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const KIND: &str = "AWS::SNS::Topic";

pub struct Fixture {
    pub work: tempfile::TempDir,
    pub database: PathBuf,
    pub report: PathBuf,
    pub differences: PathBuf,
    pub oracle: Option<PathBuf>,
    pub account: Option<String>,
}

impl Fixture {
    pub async fn new(stored: &[&str], capability: &str, oracle: &str) -> TestResult<Self> {
        let work = tempfile::tempdir()?;
        let database = work.path().join("inventory.sqlite");
        let rows = stored
            .iter()
            .map(|id| store::resource("123456789012", "us-east-1", KIND, id))
            .collect::<TestResult<Vec<_>>>()?;
        store::sqlite(&database, &rows).await?;
        configure(work.path(), capability, oracle)?;
        Ok(Self {
            database,
            report: work.path().join("check.report"),
            differences: work.path().join("differences.tsv"),
            account: None,
            oracle: Some(work.path().join("oracle.tsv")),
            work,
        })
    }

    pub fn command(
        &self,
        server: &Server,
        database: Option<&str>,
    ) -> TestResult<tokio::process::Command> {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
        command
            .args(["verify", "cloud-control", "--database"])
            .arg(database.map_or_else(
                || format!("sqlite://{}", self.database.display()),
                str::to_owned,
            ))
            .arg("--endpoint")
            .arg(server.endpoint()?)
            .arg("--ca-bundle")
            .arg(&server.ca)
            .args(["--region", "us-east-1", "--report"])
            .arg(&self.report)
            .arg("--universal")
            .arg(self.work.path().join("universal.report"))
            .arg("--differences")
            .arg(&self.differences)
            .env_clear()
            .env("AWS_ACCESS_KEY_ID", "TEST")
            .env("AWS_SECRET_ACCESS_KEY", "TEST")
            .env("AWS_EC2_METADATA_DISABLED", "true")
            .kill_on_drop(true);
        if let Some(account) = &self.account {
            command.arg("--account").arg(account);
        }
        if let Some(oracle) = &self.oracle {
            command.arg("--oracle").arg(oracle);
        } else {
            command.arg("--seeds").arg(self.work.path().join("seeds"));
        }
        Ok(command)
    }
}

fn configure(work: &Path, capability: &str, oracle: &str) -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bytes = std::fs::read(root.join("catalog/aws-api.catalog"))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let rows: Vec<_> = catalog.coverage().iter().filter_map(|entry| entry.cfn_type.as_deref())
        .map(|name| json!({"type_name": name, "capability": if name == KIND { capability } else { "unsupported" }})).collect();
    let universal: Value = json!({"schema_version":1,"resource_types":rows});
    std::fs::write(
        work.join("universal.report"),
        serde_json::to_vec(&universal)?,
    )?;
    std::fs::write(work.join("oracle.tsv"), oracle)?;
    std::fs::write(work.join("differences.tsv"), "# type\trule\treason\n")?;
    Ok(())
}

pub async fn execute(
    fixture: &Fixture,
    replies: &[(u16, &str)],
    database: Option<&str>,
) -> TestResult<(std::process::Output, Vec<String>)> {
    let server = Server::new().await?;
    let mut command = fixture.command(&server, database)?;
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
    Ok((output, requests))
}

pub fn topic_report(output: &[u8]) -> TestResult<String> {
    Ok(String::from_utf8(output.to_owned())?
        .lines()
        .filter(|line| line.starts_with("AWS::SNS::Topic\t"))
        .map(|line| format!("{line}\n"))
        .collect())
}

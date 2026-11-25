use super::TestResult;
use inventory_harness::emulator::Emulator;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub struct Native {
    work: tempfile::TempDir,
    emulator: Emulator,
    credentials: Vec<(&'static str, String)>,
}

impl Native {
    pub fn start() -> TestResult<Self> {
        let binary = std::env::var_os("INVENTORY_EMULATOR_BINARY")
            .map(PathBuf::from)
            .ok_or("native probe requires INVENTORY_EMULATOR_BINARY")?;
        let work = tempfile::tempdir()?;
        let emulator = Emulator::start(&binary, work.path(), 300)?;
        let credentials = credentials(emulator.credentials())?;
        Ok(Self {
            work,
            emulator,
            credentials,
        })
    }

    pub async fn seed(&self) -> TestResult {
        let mut command = self.command();
        command.args(["seed", "run"]);
        self.configure(&mut command);
        command
            .arg("--seeds")
            .arg(workspace().join("crates/harness/seeds"))
            .arg("--oracle")
            .arg(self.work.path().join("seed-oracle.tsv"))
            .arg("--matrix")
            .arg(self.work.path().join("seed-matrix.report"));
        check(command.output().await?)
    }

    pub async fn probe(&self) -> TestResult<String> {
        let report = self.work.path().join("probe.report");
        let mut command = self.command();
        command.arg("probe");
        self.configure(&mut command);
        command.arg("--report").arg(&report);
        check(command.output().await?)?;
        Ok(tokio::fs::read_to_string(report).await?)
    }

    pub fn stop(self) -> TestResult {
        self.emulator.stop()?;
        Ok(())
    }

    fn command(&self) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
        command
            .env_remove("AWS_SESSION_TOKEN")
            .envs(self.credentials.iter().map(|(key, value)| (*key, value)))
            .kill_on_drop(true);
        command
    }

    fn configure(&self, command: &mut tokio::process::Command) {
        command
            .arg("--catalog")
            .arg(workspace().join("catalog/aws-api.catalog"))
            .arg("--endpoint")
            .arg(self.emulator.endpoint())
            .arg("--ca-bundle")
            .arg(self.emulator.ca_bundle())
            .arg("--region")
            .arg("us-east-1");
    }
}

pub fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn credentials(path: &Path) -> TestResult<Vec<(&'static str, String)>> {
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(65_537)
        .read_to_string(&mut text)?;
    if text.len() > 65_536 {
        return Err("native credentials exceed their bound".into());
    }
    let mut values = Vec::new();
    for (key, value) in text.lines().filter_map(|line| line.split_once('=')) {
        let key = match key.trim() {
            "aws_access_key_id" => "AWS_ACCESS_KEY_ID",
            "aws_secret_access_key" => "AWS_SECRET_ACCESS_KEY",
            "aws_session_token" => "AWS_SESSION_TOKEN",
            _ => continue,
        };
        if values.len() == 3 {
            return Err("native credentials exceed three fields".into());
        }
        values.push((key, value.trim().to_owned()));
    }
    Ok(values)
}

fn check(output: std::process::Output) -> TestResult {
    if !output.status.success() {
        return Err(format!(
            "native CLI {:?}: {}",
            output.status.code(),
            String::from_utf8(output.stderr)?
        )
        .into());
    }
    Ok(())
}

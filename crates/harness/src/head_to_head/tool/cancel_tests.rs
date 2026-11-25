use super::tests::descendant_stopped;
use super::*;
use crate::{
    head_to_head::{StopSignals, until_stop, write_outcome},
    postgres::{
        CleanupFailures, TestDatabase,
        worker::{Actions, CreateOutcome},
    },
};
use rustix::process::Signal;
type TestResult = Result<(), Box<dyn std::error::Error>>;

struct FileDatabase(PathBuf);
impl Actions for FileDatabase {
    async fn create(&self) -> CreateOutcome {
        match std::fs::write(&self.0, "owned database handle") {
            Ok(()) => CreateOutcome::Created { close: Ok(()) },
            Err(error) => CreateOutcome::NotCreated(error.into()),
        }
    }
    async fn remove(&self) -> Result<(), HarnessError> {
        std::fs::remove_file(&self.0)?;
        Ok(())
    }
}

#[tokio::test]
async fn sample_signal_fixture() -> TestResult {
    let Some(root) = std::env::var_os("H2H_SAMPLE_SIGNAL_ROOT").map(PathBuf::from) else {
        return Ok(());
    };
    let mut signals = StopSignals::register()?;
    let failures = CleanupFailures::default();
    let database = TestDatabase::create_with_actions(
        "postgres://postgres@127.0.0.1:1/postgres",
        &failures,
        FileDatabase(root.join("database-live")),
    )
    .await?;
    let mut command = Command::new("sh");
    command.args([
        "-c",
        "sleep 60 & printf '%s\\n%s\\n' \"$$\" \"$!\" > state.tmp; mv state.tmp state; wait",
    ]);
    let run = async {
        let _database = database;
        execute_process(command, &root, "collect").await?;
        Ok(serde_json::json!({"comparison_ready": true}))
    };
    let result = match until_stop(run, signals.received()).await {
        std::ops::ControlFlow::Break(error) => Err(error),
        std::ops::ControlFlow::Continue(result) => result,
    };
    let error = write_outcome(&root, result, &failures)
        .err()
        .ok_or("expected cancellation")?;
    assert_eq!(error.to_string(), "head-to-head run interrupted");
    Err(error.into())
}

struct FixtureChild(ChildOwner);

impl FixtureChild {
    fn signal(&self, signal: Signal) -> Result<(), HarnessError> {
        if self.0.exited()?.is_some() {
            return Ok(());
        }
        match rustix::process::kill_process(self.0.pid(), signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(error) => Err(std::io::Error::from(error).into()),
        }
    }
}

impl Drop for FixtureChild {
    fn drop(&mut self) {
        let result = self
            .signal(Signal::TERM)
            .map_err(Box::<dyn std::error::Error>::from)
            .and_then(|()| wait_for(|| Ok(self.0.exited()?.is_some())));
        if let Err(error) = result {
            eprintln!("fixture shutdown failed: {error}");
        }
    }
}

fn wait_for(mut predicate: impl FnMut() -> Result<bool, Box<dyn std::error::Error>>) -> TestResult {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(35))
        .ok_or("invalid deadline")?;
    for _ in 0..3501 {
        if predicate()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("fixture readiness deadline elapsed".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("fixture readiness polling budget exhausted".into())
}

fn spawn_sample_fixture(root: &Path) -> Result<FixtureChild, HarnessError> {
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command
        .args([
            "--exact",
            "head_to_head::tool::cancel_tests::sample_signal_fixture",
            "--nocapture",
        ])
        .env("H2H_SAMPLE_SIGNAL_ROOT", root)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    Ok(FixtureChild(ChildOwner::spawn(&mut command)?))
}

async fn signal_sample(signal: Signal) -> TestResult {
    let root = tempfile::tempdir()?;
    let child = spawn_sample_fixture(root.path())?;
    wait_for(|| Ok(root.path().join("state").is_file()))?;
    let state = std::fs::read_to_string(root.path().join("state"))?;
    let mut pids = state.lines();
    let leader = pids.next().ok_or("missing leader")?.parse::<i32>()?;
    let descendant = pids.next().ok_or("missing descendant")?.parse::<i32>()?;
    assert_eq!(
        std::fs::read_to_string(root.path().join("database-live"))?,
        "owned database handle"
    );
    child.signal(signal)?;
    wait_for(|| Ok(child.0.exited()?.is_some()))?;
    let status = child.0.exited()?.ok_or("missing fixture status")?;
    assert_eq!(status.exit_status(), Some(101));
    assert!(descendant_stopped(leader).await?);
    assert!(descendant_stopped(descendant).await?);
    assert!(!root.path().join("database-live").exists());
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.path().join("report.json"))?)?;
    assert_eq!(
        report,
        serde_json::json!({"comparison_ready": false, "error": "head-to-head run interrupted", "cleanup_failures": []})
    );
    Ok(())
}

#[tokio::test]
async fn given_active_sample_when_test_guard_drops_then_harness_cleans_its_own_sample() -> TestResult
{
    let root = tempfile::tempdir()?;
    let child = spawn_sample_fixture(root.path())?;
    let state = root.path().join("state");
    wait_for(|| Ok(state.is_file()))?;
    let state = std::fs::read_to_string(state)?;
    let mut pids = state.lines();
    let leader = pids.next().ok_or("missing leader")?.parse::<i32>()?;
    let descendant = pids.next().ok_or("missing descendant")?.parse::<i32>()?;

    drop(child);

    assert!(descendant_stopped(leader).await?);
    assert!(descendant_stopped(descendant).await?);
    assert!(!root.path().join("database-live").exists());
    Ok(())
}

#[tokio::test]
async fn given_active_sample_when_sigint_arrives_then_process_group_and_database_handle_are_removed()
-> TestResult {
    signal_sample(Signal::INT).await
}
#[tokio::test]
async fn given_active_sample_when_sigterm_arrives_then_process_group_and_database_handle_are_removed()
-> TestResult {
    signal_sample(Signal::TERM).await
}
#[tokio::test]
async fn given_active_sample_when_sighup_arrives_then_process_group_and_database_handle_are_removed()
-> TestResult {
    signal_sample(Signal::HUP).await
}
#[tokio::test]
async fn given_active_sample_when_sigquit_arrives_then_process_group_and_database_handle_are_removed()
-> TestResult {
    signal_sample(Signal::QUIT).await
}

#[tokio::test]
async fn credential_fixture() -> TestResult {
    if std::env::var_os("H2H_CREDENTIAL_FIXTURE").is_some() {
        for name in ["AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN", "AWS_PROFILE"] {
            assert_eq!(std::env::var(name)?, "parent-only-sentinel");
        }
    }
    let root = tempfile::tempdir()?;
    let command = external_command(
        &["sh".into(), "-c".into(), "printf '%s\\n' \"${AWS_SECRET_ACCESS_KEY-unset}\" \"${AWS_SESSION_TOKEN-unset}\" \"$AWS_PROFILE\"".into()],
        &BTreeMap::from([("AWS_PROFILE".into(), "issued-profile".into())]), &BTreeMap::new(),
    )?;
    let output = execute_process(command, root.path(), "credentials").await?;
    assert_eq!(output.stdout, "unset\nunset\nissued-profile\n");
    Ok(())
}

#[test]
fn given_parent_aws_credentials_when_external_command_runs_then_sentinels_are_not_inherited()
-> TestResult {
    let output = std::process::Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "head_to_head::tool::cancel_tests::credential_fixture",
            "--nocapture",
        ])
        .env("H2H_CREDENTIAL_FIXTURE", "1")
        .env("AWS_SECRET_ACCESS_KEY", "parent-only-sentinel")
        .env("AWS_SESSION_TOKEN", "parent-only-sentinel")
        .env("AWS_PROFILE", "parent-only-sentinel")
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)?.contains("1 passed; 0 failed"));
    Ok(())
}

#[test]
fn given_inherited_ignored_sigchld_when_tool_exits_then_leader_can_be_reaped() -> TestResult {
    let output = std::process::Command::new("/usr/bin/env")
        .arg("--ignore-signal=CHLD")
        .arg(std::env::current_exe()?)
        .args([
            "--exact",
            "head_to_head::tool::cancel_tests::credential_fixture",
            "--nocapture",
        ])
        .env_remove("H2H_CREDENTIAL_FIXTURE")
        .output()?;

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(String::from_utf8(output.stdout)?.contains("1 passed; 0 failed"));
    Ok(())
}

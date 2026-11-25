use super::*;
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions};
use std::{
    os::unix::{fs::PermissionsExt, process::CommandExt},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn snapshot_signal_fixture() -> TestResult {
    let Some(root) = std::env::var_os("H2H_SNAPSHOT_SIGNAL_ROOT").map(PathBuf::from) else {
        return Ok(());
    };
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let args = Args {
        work: root.join("work"),
        ours: "/usr/bin/true".into(),
        tools: vec![("other".into(), root.join("tool.toml"))],
        runs: 1,
        catalog: root.join("catalog"),
        seeds: root.join("seeds"),
        services: vec!["s3".into()],
    };
    let marker = root.join("snapshot-started");
    let run = execute_with_snapshot(
        args,
        "postgres://postgres@127.0.0.1:1/postgres",
        move |args| {
            snapshot_dataset(args, |path| {
                std::fs::write(&marker, "copying")?;
                gate.recv_timeout(Duration::from_secs(5))
                    .map_err(|_| HarnessError::invalid("snapshot read gate elapsed"))?;
                crate::files::read_bytes(path, 1_048_576)
            })
        },
    );
    let release_on_signal = async {
        signal.recv().await.ok_or("snapshot signal stream closed")?;
        release
            .send(())
            .map_err(|_| "snapshot reader already stopped")?;
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    let (result, released) = tokio::join!(run, release_on_signal);
    let error = result.err().ok_or("expected snapshot cancellation")?;
    std::fs::write(root.join("outcome"), error.to_string())?;
    assert_eq!(error.to_string(), "head-to-head run interrupted");
    released?;
    Err(error.into())
}

struct SnapshotChild {
    child: Child,
    pid: Pid,
}
impl Drop for SnapshotChild {
    fn drop(&mut self) {
        if let Err(error) = rustix::process::kill_process_group(self.pid, Signal::KILL)
            && error != rustix::io::Errno::SRCH
        {
            eprintln!("snapshot fixture cleanup failed: {error}");
        }
        if let Err(error) = self.child.wait() {
            eprintln!("snapshot fixture reap failed: {error}");
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
            return Err("snapshot fixture deadline elapsed".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("snapshot fixture polling budget exhausted".into())
}

#[test]
fn given_snapshot_in_progress_when_signal_arrives_then_no_stack_starts_and_exit_is_failure()
-> TestResult {
    let root = tempfile::tempdir()?;
    std::fs::create_dir(root.path().join("seeds"))?;
    std::fs::write(root.path().join("seeds/s3.toml"), "resources = []\n")?;
    std::fs::write(root.path().join("catalog"), "")?;
    std::fs::write(
        root.path().join("tool.toml"),
        "services = ['s3']\nversion = ['true']\nreset = ['true']\nrun = ['true']\ncounts = ['true']\n",
    )?;
    let runtime = root.path().join("runtime.sh");
    std::fs::write(
        &runtime,
        format!(
            "#!/bin/sh\ntouch '{}'\nexit 1\n",
            root.path().join("stack-started").display()
        ),
    )?;
    std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o755))?;
    let child = Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "head_to_head::snapshot_tests::snapshot_signal_fixture",
            "--nocapture",
        ])
        .env("H2H_SNAPSHOT_SIGNAL_ROOT", root.path())
        .env("INVENTORY_EMULATOR_BINARY", runtime)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    let pid = Pid::from_raw(i32::try_from(child.id())?).ok_or("invalid snapshot fixture PID")?;
    let _child = SnapshotChild { child, pid };
    wait_for(|| Ok(root.path().join("snapshot-started").is_file()))?;
    rustix::process::kill_process(pid, Signal::INT)?;
    wait_for(|| {
        Ok(rustix::process::waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )?
        .is_some())
    })?;
    let status = rustix::process::waitid(
        WaitId::Pid(pid),
        WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
    )?
    .ok_or("missing fixture status")?;
    assert_eq!(status.exit_status(), Some(101));
    assert_eq!(
        std::fs::read_to_string(root.path().join("outcome"))?,
        "head-to-head run interrupted"
    );
    assert!(!root.path().join("stack-started").exists());
    assert_eq!(
        std::fs::read_to_string(root.path().join("work/dataset/s3.toml"))?,
        "resources = []\n"
    );
    Ok(())
}

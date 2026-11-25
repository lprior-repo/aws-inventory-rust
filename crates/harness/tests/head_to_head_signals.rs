use rustix::process::{Pid, Signal, WaitId, WaitIdOptions};
use std::{
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CLI_TIMEOUT: Duration = Duration::from_secs(60);
const UNREACHABLE_POSTGRES: &str = "postgres://postgres@127.0.0.1:1/postgres";
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(5);

struct OwnedChild {
    child: Child,
    pid: Pid,
}

impl OwnedChild {
    fn spawn(mut command: Command) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()?;
        let installed = {
            let _entered = runtime.enter();
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::child()).map(drop)
        };
        runtime.shutdown_background();
        installed?;
        let child = command.process_group(0).spawn()?;
        let pid = Pid::from_child(&child);
        Ok(Self { child, pid })
    }

    fn signal(&self, signal: Signal) -> TestResult {
        if self.exited()? {
            return Ok(());
        }
        match rustix::process::kill_process(self.pid, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn exited(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let flags = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;
        Ok(rustix::process::waitid(WaitId::Pid(self.pid), flags)?.is_some())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let graceful = self
            .signal(Signal::TERM)
            .and_then(|()| wait_for(CLI_TIMEOUT, || self.exited()));
        if let Err(error) = graceful {
            eprintln!("signal-test shutdown failed: {error}");
            if !matches!(self.exited(), Ok(false)) {
                return;
            }
            match rustix::process::kill_process_group(self.pid, Signal::KILL) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                Err(error) => {
                    eprintln!("signal-test process group cleanup failed: {error}");
                    return;
                }
            }
        }
        if let Err(error) = self.child.wait() {
            eprintln!("signal-test leader cleanup failed: {error}");
        }
    }
}

fn executable(path: &Path, script: &str) -> TestResult {
    std::fs::write(path, script)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn startup_fixtures(root: &Path, graph_finish: &str) -> TestResult {
    let version = inventory_harness::emulator::PINNED_VERSION;
    let commit = inventory_harness::emulator::PINNED_COMMIT;
    executable(
        &root.join("emulator.sh"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = --version ]; then echo '{version} {commit}'; exit 0; fi\n\
             sleep 90 &\nprintf '%s\\n%s\\n' \"$$\" \"$!\" > \"$HOME/runtime.tmp\"\n\
             mv \"$HOME/runtime.tmp\" \"$HOME/emulator.pids\"\n\
             printf '%s\\n' '{{\"fields\":{{\"endpoint_url\":\"https://127.0.0.1:1\"}}}}'\nwait\n"
        ),
    )?;
    executable(
        &root.join("redis.sh"),
        &format!(
            "#!/bin/sh\nwhile [ \"$1\" != --dir ]; do shift; done\nroot=$2\n\
             sleep 90 &\nprintf '%s\\n%s\\n' \"$$\" \"$!\" > \"$root/runtime.tmp\"\n\
             mv \"$root/runtime.tmp\" \"$root/graph.pids\"\n{graph_finish}\n"
        ),
    )?;
    std::fs::write(root.join("module.so"), "")?;
    std::fs::write(root.join("catalog"), "")?;
    std::fs::write(
        root.join("other.toml"),
        "services = ['s3']\nversion = ['true']\nreset = ['true']\nrun = ['true']\ncounts = ['true']\n",
    )?;
    Ok(())
}

fn cli(root: &Path, collector: &Path) -> Result<Command, Box<dyn std::error::Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command
        .args(["head-to-head", "--work"])
        .arg(root.join("work"))
        .arg("--ours")
        .arg(collector)
        .arg("--catalog")
        .arg(root.join("catalog"))
        .arg("--tool")
        .arg(format!("other={}", root.join("other.toml").display()))
        .args(["--runs", "1", "--service", "s3"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(std::fs::File::create(root.join("stderr"))?));
    Ok(command)
}

fn wait_for(
    timeout: Duration,
    mut predicate: impl FnMut() -> Result<bool, Box<dyn std::error::Error>>,
) -> TestResult {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or("invalid deadline")?;
    for _ in 0..6001 {
        if predicate()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("signal-test deadline elapsed".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("signal-test polling budget exhausted".into())
}

fn wait_for_marker(child: &OwnedChild, marker: &Path, stderr: &Path) -> TestResult {
    wait_for(CLI_TIMEOUT, || {
        if marker.is_file() {
            return Ok(true);
        }
        if child.exited()? {
            let log = std::fs::read_to_string(stderr).unwrap_or_default();
            return Err(format!("CLI exited before {} appeared:\n{log}", marker.display()).into());
        }
        Ok(false)
    })
}

fn stopped(pid: i32) -> Result<bool, Box<dyn std::error::Error>> {
    match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => Ok(stat
            .rsplit_once(") ")
            .and_then(|(_, tail)| tail.chars().next())
            .is_some_and(|state| matches!(state, 'Z' | 'X'))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error.into()),
    }
}

fn startup_cli(root: &Path) -> Result<OwnedChild, Box<dyn std::error::Error>> {
    let mut command = cli(root, Path::new("/usr/bin/true"))?;
    command
        .env("INVENTORY_POSTGRES_URL", UNREACHABLE_POSTGRES)
        .env("INVENTORY_EMULATOR_BINARY", root.join("emulator.sh"))
        .env("INVENTORY_REDIS_SERVER", root.join("redis.sh"))
        .env("INVENTORY_FALKOR_MODULE", root.join("module.so"));
    OwnedChild::spawn(command)
}

fn runtime_pids(path: &Path) -> Result<[i32; 2], Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    let mut lines = text.lines();
    Ok([
        lines.next().ok_or("missing runtime leader PID")?.parse()?,
        lines.next().ok_or("missing runtime worker PID")?.parse()?,
    ])
}

fn assert_runtime_groups_stopped(work: &Path) -> TestResult {
    let [graph, graph_worker] = runtime_pids(&work.join("graph.pids"))?;
    let [emulator, emulator_worker] = runtime_pids(&work.join("emulator/home/emulator.pids"))?;
    wait_for(CLEANUP_TIMEOUT, || {
        Ok(stopped(graph)?
            && stopped(graph_worker)?
            && stopped(emulator)?
            && stopped(emulator_worker)?)
    })
}

fn startup_signal(signal: Signal) -> TestResult {
    let root = tempfile::tempdir()?;
    startup_fixtures(root.path(), "wait")?;
    let child = startup_cli(root.path())?;
    let work = root.path().join("work");
    wait_for_marker(
        &child,
        &work.join("graph.pids"),
        &root.path().join("stderr"),
    )?;

    child.signal(signal)?;
    wait_for(CLI_TIMEOUT, || child.exited())?;

    let stderr = std::fs::read_to_string(root.path().join("stderr"))?;
    let status = rustix::process::waitid(
        WaitId::Pid(child.pid),
        WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
    )?
    .ok_or("missing CLI exit status")?;
    assert_eq!(status.exit_status(), Some(1));
    assert!(
        stderr.contains("head-to-head run interrupted during startup"),
        "{stderr}"
    );
    assert_runtime_groups_stopped(&work)?;
    drop(child);
    Ok(())
}

#[test]
fn given_graph_wrapper_exits_before_readiness_when_startup_fails_then_both_groups_are_stopped()
-> TestResult {
    let root = tempfile::tempdir()?;
    startup_fixtures(root.path(), "exit 7")?;

    let child = startup_cli(root.path())?;
    wait_for(CLI_TIMEOUT, || child.exited())?;

    let status = rustix::process::waitid(
        WaitId::Pid(child.pid),
        WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
    )?
    .ok_or("missing CLI exit status")?;
    assert_eq!(status.exit_status(), Some(1));
    assert_runtime_groups_stopped(&root.path().join("work"))
}

#[test]
fn given_startup_in_progress_when_sigint_arrives_then_owned_children_are_stopped() -> TestResult {
    startup_signal(Signal::INT)
}

#[test]
fn given_startup_in_progress_when_sigterm_arrives_then_owned_children_are_stopped() -> TestResult {
    startup_signal(Signal::TERM)
}

#[test]
fn given_startup_in_progress_when_sighup_arrives_then_owned_children_are_stopped() -> TestResult {
    startup_signal(Signal::HUP)
}

#[test]
fn given_startup_in_progress_when_sigquit_arrives_then_owned_children_are_stopped() -> TestResult {
    startup_signal(Signal::QUIT)
}

async fn database_exists(admin: &str, database: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use sqlx::{
        Connection,
        postgres::{PgConnectOptions, PgConnection},
    };
    use std::str::FromStr;
    let options = PgConnectOptions::from_str(database)?;
    let name = options
        .get_database()
        .ok_or("missing sample database name")?;
    let mut connection = PgConnection::connect(admin).await?;
    let exists = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(name)
        .fetch_one(&mut connection)
        .await?;
    connection.close().await?;
    Ok(exists)
}

fn native_fixtures(root: &Path) -> TestResult {
    let catalog = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog");
    std::os::unix::fs::symlink(catalog, root.join("catalog"))?;
    std::fs::write(
        root.join("other.toml"),
        "services = ['s3']\nversion = ['true']\nreset = ['true']\nrun = ['true']\ncounts = ['true']\n",
    )?;
    executable(
        &root.join("collector.sh"),
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo cancellation-fixture; exit 0; fi\nsleep 90 &\nprintf '%s\\n%s\\n%s\\n' \"$$\" \"$!\" \"$INVENTORY_SWEEP_DATABASE\" > state.tmp\nmv state.tmp state\nwait\n",
    )?;
    Ok(())
}

async fn sample_signal(signal: Signal) -> TestResult {
    let parent =
        std::env::var_os("INVENTORY_H2H_TEST_WORK").map_or_else(std::env::temp_dir, PathBuf::from);
    std::fs::create_dir_all(&parent)?;
    let root = tempfile::tempdir_in(parent)?;
    native_fixtures(root.path())?;
    let admin = inventory_harness::postgres::admin_url()?;
    let mut command = cli(root.path(), &root.path().join("collector.sh"))?;
    command.env("INVENTORY_POSTGRES_URL", &admin);
    let child = OwnedChild::spawn(command)?;
    let state = root.path().join("work/run-0-ours/state");
    wait_for_marker(&child, &state, &root.path().join("stderr"))?;
    let state = std::fs::read_to_string(state)?;
    let mut fields = state.lines();
    let leader = fields.next().ok_or("missing sample PID")?.parse::<i32>()?;
    let descendant = fields
        .next()
        .ok_or("missing descendant PID")?
        .parse::<i32>()?;
    let database = fields.next().ok_or("missing sample database URL")?;
    let existed = database_exists(&admin, database).await?;
    child.signal(signal)?;
    wait_for(CLI_TIMEOUT, || child.exited())?;
    wait_for(CLEANUP_TIMEOUT, || {
        Ok(stopped(leader)? && stopped(descendant)?)
    })?;
    let remains = database_exists(&admin, database).await?;
    let stderr = std::fs::read_to_string(root.path().join("stderr"))?;
    assert!(existed, "sample database did not exist before cancellation");
    assert!(!remains, "cancelled sample database still exists");
    assert!(stderr.contains("head-to-head run interrupted"), "{stderr}");
    drop(child);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
async fn given_active_postgres_sample_when_sigint_arrives_then_database_and_children_are_removed()
-> TestResult {
    sample_signal(Signal::INT).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
async fn given_active_postgres_sample_when_sigterm_arrives_then_database_and_children_are_removed()
-> TestResult {
    sample_signal(Signal::TERM).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
async fn given_active_postgres_sample_when_sighup_arrives_then_database_and_children_are_removed()
-> TestResult {
    sample_signal(Signal::HUP).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and installed emulator/FalkorDB runtimes"]
async fn given_active_postgres_sample_when_sigquit_arrives_then_database_and_children_are_removed()
-> TestResult {
    sample_signal(Signal::QUIT).await
}

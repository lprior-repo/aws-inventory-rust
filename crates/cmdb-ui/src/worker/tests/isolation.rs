//! A hung App case is a reaped child process, never a detached Rust thread.
use super::real::fixtures::TestResult;
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    process::{Child, Command, ExitStatus},
    time::{Duration, Instant},
};

const CHILD_CASE: &str = "CMDB_UI_CHILD_TEST_CASE";
const GRAPH_LEDGER: &str = "CMDB_UI_CHILD_GRAPH_LEDGER";
const FIXTURE_ROOT: &str = "CMDB_UI_CHILD_FIXTURE_ROOT";

struct ChildOwner(Child);

impl Drop for ChildOwner {
    fn drop(&mut self) {
        match self.0.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(error) => eprintln!("failed to inspect owned UI test child: {error}"),
        }
        if let Err(error) = self.0.kill() {
            eprintln!("failed to terminate owned UI test child: {error}");
        }
        if let Err(error) = self.0.wait() {
            eprintln!("failed to reap owned UI test child: {error}");
        }
    }
}

fn await_child(child: &mut ChildOwner) -> TestResult<ExitStatus> {
    let deadline = Instant::now() + Duration::from_secs(45);
    while Instant::now() < deadline {
        if let Some(status) = child.0.try_wait()? {
            return Ok(status);
        }
        std::thread::yield_now();
    }
    child.0.kill()?;
    child.0.wait()?;
    Err("App case exceeded its 45-second process deadline; child terminated and reaped".into())
}

pub(crate) fn supervised(job: impl FnOnce() -> TestResult) -> TestResult {
    let current = std::thread::current();
    let name = current
        .name()
        .ok_or("UI test requires a named harness thread")?;
    if std::env::var(CHILD_CASE).is_ok_and(|child| child == name) {
        return job();
    }
    let directory = tempfile::tempdir()?;
    let ledger = directory.path().join("owned-graphs");
    let mut child = ChildOwner(
        Command::new(std::env::current_exe()?)
            .args(["--exact", name, "--nocapture", "--test-threads=1"])
            .env(CHILD_CASE, name)
            .env(GRAPH_LEDGER, &ledger)
            .env(FIXTURE_ROOT, directory.path())
            .spawn()?,
    );
    let result = await_child(&mut child);
    // Reap before deleting graph fixtures, including on wait/kill failures.
    drop(child);
    let cleanup = cleanup_ledger(&ledger);
    match (result, cleanup) {
        (Ok(status), Ok(())) if status.success() => Ok(()),
        (Ok(status), Ok(())) => Err(format!("isolated UI test {name} exited with {status}").into()),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => {
            Err(format!("{error}; owned graph cleanup also failed: {cleanup}").into())
        }
    }
}

pub(crate) fn supervised_async<F: Future<Output = TestResult>>(
    job: impl FnOnce() -> F,
) -> TestResult {
    supervised(|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(job())
    })
}

pub(crate) fn temporary_directory() -> TestResult<tempfile::TempDir> {
    Ok(match std::env::var_os(FIXTURE_ROOT) {
        Some(root) => tempfile::tempdir_in(root)?,
        None => tempfile::tempdir()?,
    })
}

pub(crate) fn register_graph(name: &str) -> TestResult {
    let Some(path) = std::env::var_os(GRAPH_LEDGER) else {
        return Ok(());
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{name}")?;
    Ok(())
}

fn cleanup_ledger(path: &Path) -> TestResult {
    let names = match std::fs::read_to_string(path) {
        Ok(names) => names,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if names.len() > 65_536 || names.lines().count() > 64 {
        return Err("owned native graph ledger exceeded its bound".into());
    }
    for name in names.lines() {
        if !name.starts_with("ui_worker_")
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("owned native graph ledger contains an invalid graph name".into());
        }
        delete_graph(name)?;
    }
    Ok(())
}

pub(crate) fn delete_graph(name: &str) -> TestResult {
    let endpoint = std::env::var("CMDB_VIEW_FALKOR_ENDPOINT")
        .map_or_else(|_| "redis://127.0.0.1:6379".to_owned(), |value| value);
    let client = redis::Client::open(endpoint)?;
    let limit = Duration::from_secs(2);
    let mut connection = client.get_connection_with_timeout(limit)?;
    connection.set_read_timeout(Some(limit))?;
    connection.set_write_timeout(Some(limit))?;
    let exists: bool = redis::cmd("EXISTS").arg(name).query(&mut connection)?;
    if exists {
        redis::cmd("GRAPH.DELETE")
            .arg(name)
            .query::<redis::Value>(&mut connection)?;
    }
    Ok(())
}

#![cfg(target_os = "linux")]

#[path = "../../cmdb-view/tests/postgres_support/mod.rs"]
#[allow(dead_code)]
mod pg;

use pg::{Fixture, TestResult};
use std::{
    collections::{BTreeSet, HashSet},
    fs,
    io::{self, Read},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const EXPECTED_RESOURCES: u64 = 4;
const EXPECTED_EDGES: u64 = 2;
const HARNESS_SECONDS: &str = "3";
const CAPTURE_COMPLETE: &str = r#"capture_status="complete""#;
const DEADLINE: Duration = Duration::from_secs(120);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(50);
const TCP_LISTEN: &str = "0A";
const TCP_TABLES: [&str; 2] = ["/proc/net/tcp", "/proc/net/tcp6"];

struct ChildOwner(Child);

impl Drop for ChildOwner {
    fn drop(&mut self) {
        if let Ok(None) = self.0.try_wait() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

#[derive(Default)]
struct Observed {
    attributed: HashSet<u64>,
    listeners: BTreeSet<String>,
}

fn drain<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn collect(handle: JoinHandle<io::Result<Vec<u8>>>) -> TestResult<Vec<u8>> {
    let bytes = handle.join().map_err(|_| "pipe reader thread panicked")??;
    Ok(bytes)
}

fn socket_inode(target: &Path) -> TestResult<Option<u64>> {
    let text = target.to_string_lossy();
    let Some(inode) = text.strip_prefix("socket:[") else {
        return Ok(None);
    };
    let parsed = inode
        .strip_suffix(']')
        .and_then(|digits| digits.parse::<u64>().ok())
        .ok_or_else(|| format!("malformed socket link: {text}"))?;
    Ok(Some(parsed))
}

fn socket_inodes(pid: u32) -> TestResult<HashSet<u64>> {
    let mut inodes = HashSet::new();
    for entry in fs::read_dir(format!("/proc/{pid}/fd"))? {
        match fs::read_link(entry?.path()) {
            Ok(target) => inodes.extend(socket_inode(&target)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(inodes)
}

fn tcp_rows(table: &str) -> TestResult<Vec<(u64, bool, &str)>> {
    let mut lines = table.lines();
    let header = lines.next().ok_or("empty TCP table")?;
    if !header.trim_start().starts_with("sl ") {
        return Err("unexpected TCP table header".into());
    }
    let mut rows = Vec::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [_, _, _, state, _, _, _, _, _, inode, ..] = fields.as_slice() else {
            return Err(format!("malformed TCP row: {line}").into());
        };
        let inode = inode
            .parse::<u64>()
            .map_err(|error| format!("malformed TCP inode in `{line}`: {error}"))?;
        rows.push((inode, *state == TCP_LISTEN, line));
    }
    Ok(rows)
}

fn observe(pid: u32, observed: &mut Observed) -> TestResult {
    let inodes = socket_inodes(pid)?;
    for path in TCP_TABLES {
        let table = fs::read_to_string(path)?;
        for (inode, listening, line) in tcp_rows(&table)? {
            if !inodes.contains(&inode) {
                continue;
            }
            observed.attributed.insert(inode);
            if listening {
                observed.listeners.insert(line.trim().to_owned());
            }
        }
    }
    Ok(())
}

fn supervise(owner: &mut ChildOwner, pid: u32) -> TestResult<(ExitStatus, Observed)> {
    let deadline = Instant::now()
        .checked_add(DEADLINE)
        .ok_or("deadline overflow")?;
    let mut observed = Observed::default();
    loop {
        if let Some(status) = owner.0.try_wait()? {
            return Ok((status, observed));
        }
        if Instant::now() >= deadline {
            return Err("cmdb-ui exceeded its 120-second deadline; child is terminated and reaped on drop".into());
        }
        if let Err(error) = observe(pid, &mut observed) {
            return match owner.0.try_wait()? {
                Some(status) => Ok((status, observed)),
                None => Err(format!("could not sample the live child's sockets: {error}").into()),
            };
        }
        thread::sleep(SAMPLE_INTERVAL);
    }
}

fn input_counts(report: &str) -> Option<(u64, u64)> {
    let line = report
        .lines()
        .find(|line| line.starts_with("input_resources="))?;
    let mut fields = line.split_whitespace();
    let resources = fields
        .next()?
        .strip_prefix("input_resources=")?
        .parse::<u64>()
        .ok()?;
    let edges = fields
        .next()?
        .strip_prefix("input_edges=")?
        .parse::<u64>()
        .ok()?;
    Some((resources, edges))
}

fn run_frame_harness(database_url: &str) -> TestResult {
    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err("the frame harness opens a window: set DISPLAY or WAYLAND_DISPLAY".into());
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmdb-ui"))
        .arg("--database")
        .arg(database_url)
        .arg("--frame-harness")
        .arg(HARNESS_SECONDS)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = drain(child.stdout.take().ok_or("child stdout was not piped")?);
    let stderr = drain(child.stderr.take().ok_or("child stderr was not piped")?);
    let pid = child.id();
    let mut owner = ChildOwner(child);
    let (status, observed) = supervise(&mut owner, pid)?;
    let report = String::from_utf8_lossy(&collect(stdout)?).into_owned();
    let diagnostics = String::from_utf8_lossy(&collect(stderr)?).into_owned();
    if !status.success() {
        return Err(format!("cmdb-ui exited with {status}: {diagnostics}").into());
    }
    if !report.lines().any(|line| line == CAPTURE_COMPLETE) {
        return Err(format!("frame harness did not complete its capture: {report}").into());
    }
    let counts = input_counts(&report)
        .ok_or_else(|| format!("frame harness report has no input counts: {report}"))?;
    if counts != (EXPECTED_RESOURCES, EXPECTED_EDGES) {
        return Err(format!(
            "frame harness loaded {counts:?}, expected ({EXPECTED_RESOURCES}, {EXPECTED_EDGES}) from the seeded database"
        )
        .into());
    }
    if observed.attributed.is_empty() {
        return Err("no child-owned TCP socket was observed; socket attribution is unproven (INVENTORY_POSTGRES_URL must be a TCP URL)".into());
    }
    if !observed.listeners.is_empty() {
        return Err(format!(
            "UI process owns listening TCP sockets: {:?}",
            observed.listeners
        )
        .into());
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL (a TCP URL) and a graphical session (DISPLAY or WAYLAND_DISPLAY)"]
async fn given_populated_database_when_frame_harness_runs_then_ui_process_owns_no_listening_socket()
-> TestResult {
    let fixture = Fixture::new().await?;
    let outcome = run_frame_harness(fixture.url());
    let finished = fixture.finish().await;
    outcome?;
    finished
}

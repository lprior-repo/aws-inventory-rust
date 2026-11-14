mod coverage;
mod proxy;

use std::error::Error;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::LazyLock;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::Value;

use inventory_harness::emulator::Emulator;

type TestResult = Result<(), Box<dyn Error>>;

const BUCKET: &str = "rust-fixture-bucket";
const EMULATOR_HEADER: &str = ".local/runtime/unpacked/opt/aws-emulator/bin/aws-emulator";

const CHILD_DEADLINE: Duration = Duration::from_secs(180);
const CHILD_POLL: Duration = Duration::from_millis(50);

#[test]
#[ignore = "requires the pinned emulator binary and a collector build"]
fn given_owned_emulator_when_collector_scans_then_seeded_bucket_is_reported() -> TestResult {
    let emulator_binary = emulator_binary()?;
    let collector = collector_binary()?;
    let work = tempfile::tempdir()?;
    let emulator = Emulator::start(&emulator_binary, work.path(), 300)?;
    let environment = emulator.environment();

    let seeded = run(
        Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
            .args(["seed", "s3", "--bucket", BUCKET]),
        &environment,
    )?;
    assert!(
        seeded.status.success(),
        "seeding failed: {}",
        String::from_utf8_lossy(&seeded.stderr)
    );

    let database = text(&work.path().join("inventory.sqlite"))?;
    let scanned = run(
        Command::new(&collector).args([
            "scan",
            "--db",
            &database,
            "--region",
            "us-east-1",
            "--service",
            "s3",
            "--attempts",
            "1",
        ]),
        &environment,
    )?;
    assert!(
        scanned.status.success(),
        "scan failed: {}",
        String::from_utf8_lossy(&scanned.stderr)
    );
    let report: Value = serde_json::from_slice(&scanned.stdout)?;
    assert_eq!(
        report.get("status").and_then(Value::as_str),
        Some("complete")
    );
    assert_eq!(report.pointer("/api/in_flight"), Some(&Value::from(0)));
    assert!(
        report
            .pointer("/api/attempts")
            .and_then(Value::as_u64)
            .is_some_and(|at| at > 0),
        "the collector made no API call"
    );

    let scan_id = report
        .pointer("/scans/0/scan_id")
        .and_then(Value::as_str)
        .ok_or("scan report lacks its scan identity")?;
    let exported = work.path().join("inventory.json");
    let export = run(
        Command::new(&collector).args([
            "export",
            "--db",
            &database,
            "--scan-id",
            scan_id,
            "--output",
            &text(&exported)?,
            "--format",
            "json",
            "--provenance",
            "all-observations",
        ]),
        &environment,
    )?;
    assert!(
        export.status.success(),
        "export failed: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    let artifact = std::fs::read_to_string(&exported)?;
    assert!(
        artifact.contains(BUCKET),
        "the seeded bucket is absent from the exported inventory"
    );
    Ok(())
}

fn run(command: &mut Command, environment: &[(String, String)]) -> io::Result<Output> {
    for (key, _) in std::env::vars() {
        if key.starts_with("AWS_") || key.starts_with("INVENTORY_") || key == "FALKORDB_URL" {
            command.env_remove(key);
        }
    }
    for (key, value) in environment {
        command.env(key, value);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().map(drain);
    let stderr = child.stderr.take().map(drain);
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Output {
                status,
                stdout: join(stdout),
                stderr: join(stderr),
            });
        }
        if started.elapsed() >= CHILD_DEADLINE {
            let _ = child.kill();
            let _ = child.wait();
            let stderr = join(stderr);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "child exceeded {}s; stderr: {}",
                    CHILD_DEADLINE.as_secs(),
                    String::from_utf8_lossy(&stderr)
                ),
            ));
        }
        std::thread::sleep(CHILD_POLL);
    }
}

fn drain<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = pipe.read_to_end(&mut buffer);
        buffer
    })
}

fn join(handle: Option<JoinHandle<Vec<u8>>>) -> Vec<u8> {
    handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default()
}

fn scan_pages(
    database: &str,
    selection: &[&str],
    environment: &[(String, String)],
    max_pages: &str,
    page_size: &str,
) -> Result<Value, Box<dyn Error>> {
    let mut command = Command::new(collector_binary()?);
    command.args(["scan", "--db", database]);
    command.args(["--region", "us-east-1", "--attempts", "1"]);
    command.args(["--max-pages", max_pages, "--page-size", page_size]);
    command.args(["--concurrency", "8"]);
    command.args(["--per-bucket-concurrency", "2", "--rps", "100"]);
    command.args(selection);
    let output = run(&mut command, environment)?;
    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let report: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        format!("scan exited {code} without a JSON report ({error}); stderr: {stderr}")
    })?;
    match code {
        0 => {}
        2 => {
            let status = report.get("status").and_then(Value::as_str);
            if !matches!(status, Some("partial" | "cancelled")) {
                return Err(format!(
                    "scan exited 2 without a partial or cancelled report (status {status:?}); \
                     stderr: {stderr}"
                )
                .into());
            }
        }
        _ => return Err(format!("scan exited {code}: {stderr}").into()),
    }
    Ok(report)
}

fn emulator_binary() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = std::env::var_os("INVENTORY_EMULATOR_BINARY") {
        return Ok(PathBuf::from(path));
    }
    let local = workspace()?.join(EMULATOR_HEADER);
    if local.is_file() {
        return Ok(local);
    }
    Err("set INVENTORY_EMULATOR_BINARY or run `cargo run -p inventory-harness -- install .local/runtime`".into())
}

fn collector_binary() -> Result<PathBuf, Box<dyn Error>> {
    static VERIFIED: LazyLock<Result<PathBuf, String>> = LazyLock::new(verify_collector_binary);
    VERIFIED.clone().map_err(Into::into)
}

fn verify_collector_binary() -> Result<PathBuf, String> {
    let path = match std::env::var_os("INVENTORY_COLLECTOR_BINARY") {
        Some(path) => PathBuf::from(path),
        None => workspace()
            .map_err(|error| error.to_string())?
            .join("target/debug/aws-inventory"),
    };
    let rebuild = "rebuild it with `cargo build --bin aws-inventory --features aws`";
    if !path.is_file() {
        return Err(format!(
            "collector missing at {}; {rebuild}",
            path.display()
        ));
    }

    let mut command = Command::new(&path);
    command.args([
        "scan",
        "--db",
        "inventory-feature-probe.sqlite",
        "--region",
        "us-east-1",
        "--service",
        "s3",
        "--max-pages",
        "0",
    ]);
    let probe = run(
        &mut command,
        &[
            ("AWS_EC2_METADATA_DISABLED".to_owned(), "true".to_owned()),
            ("AWS_DEFAULT_REGION".to_owned(), "us-east-1".to_owned()),
        ],
    )
    .map_err(|error| {
        format!(
            "could not probe collector at {}: {error}; {rebuild}",
            path.display()
        )
    })?;
    let stderr = String::from_utf8_lossy(&probe.stderr);
    if stderr.contains("compile-time feature") {
        return Err(format!(
            "collector at {} was built without the aws feature: {}; {rebuild}",
            path.display(),
            stderr.trim()
        ));
    }
    Ok(path)
}

fn workspace() -> Result<PathBuf, Box<dyn Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn text(path: &Path) -> Result<String, Box<dyn Error>> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "path is not valid UTF-8".into())
}

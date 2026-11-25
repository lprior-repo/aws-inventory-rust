use super::{
    REGION, TestResult,
    runtime::{self, NativeRuntime},
};
use inventory_engine::inventory::{BinaryStore, run_projection};
use inventory_harness::install::graph;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::process::{Child, Command};

#[path = "collector_process.rs"]
mod collector_process;
#[path = "native_deadlines.rs"]
mod deadlines;

pub(super) const GRAPH: &str = "native_inventory_evidence";
const MAX_ROWS: usize = 10_000;

pub(super) struct NativeGraph {
    child: Child,
    endpoint: String,
    name: &'static str,
    request_budget: Duration,
    _installation: tempfile::TempDir,
}

pub(super) struct PreparedGraph {
    installation: tempfile::TempDir,
    server: PathBuf,
    module: PathBuf,
}

impl PreparedGraph {
    pub(super) fn install() -> TestResult<Self> {
        let source = std::env::var_os("AWS_INVENTORY_GRAPH_ARTIFACTS")
            .ok_or("native graph evidence requires AWS_INVENTORY_GRAPH_ARTIFACTS pinned inputs")?;
        // Compile before starting the emulator, whose deliberate idle timeout
        // must not turn a slow native build into a seeding failure.
        let installation = tempfile::tempdir()?;
        let install = installation.path().to_path_buf();
        for name in ["redis-8.10.3.tar.gz", "falkordb.so"] {
            std::fs::copy(Path::new(&source).join(name), install.join(name))?;
        }
        // Deliberately synchronous outer preparation, before emulator/tasks exist.
        // There is no cancellable blocking worker: the directory and installer
        // stay in this stack frame until the bounded native build is reaped.
        let receipt = graph::install(&install, graph::DEFAULT_JOBS)?;
        let server = path_field(&receipt, "redis_server")?;
        let module = path_field(&receipt, "falkor_module")?;
        Ok(Self {
            installation,
            server,
            module,
        })
    }

    pub(super) async fn start(self, native: &NativeRuntime) -> TestResult<NativeGraph> {
        let socket = native.work().join("falkor.sock");
        let log = std::fs::File::create(native.work().join("falkor.log"))?;
        let child = Command::new(self.server)
            .env_clear()
            .envs(native.emulator().environment())
            .args(["--bind", "127.0.0.1", "--port", "0", "--unixsocket"])
            .arg(&socket)
            .args(["--unixsocketperm", "700", "--loadmodule"])
            .arg(self.module)
            .args([
                "--appendonly",
                "no",
                "--save",
                "",
                "--daemonize",
                "no",
                "--dir",
            ])
            .arg(native.work())
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .kill_on_drop(true)
            .spawn()?;
        let mut graph = NativeGraph {
            child,
            endpoint: format!("redis+unix://{}", socket.display()),
            name: GRAPH,
            request_budget: Duration::from_secs(15),
            _installation: self.installation,
        };
        graph.await_ready().await?;
        Ok(graph)
    }
}

impl NativeGraph {
    async fn await_ready(&mut self) -> TestResult {
        self.await_ready_attempts(150).await
    }

    async fn await_ready_attempts(&mut self, attempts: usize) -> TestResult {
        let result = self.check_ready(attempts).await;
        if result.is_err() {
            self.terminate_child().await?;
        }
        result
    }

    async fn check_ready(&mut self, attempts: usize) -> TestResult {
        if !(1..=150).contains(&attempts) {
            return Err("native readiness attempt budget must be within 1..=150".into());
        }
        for remaining in (0..attempts).rev() {
            if let Some(status) = self.child.try_wait()? {
                return Err(format!("owned FalkorDB exited before readiness: {status}").into());
            }
            if let Ok(pong) = request::<String>(
                &self.endpoint,
                &redis::cmd("PING"),
                Duration::from_millis(200),
            )
            .await
            {
                assert_eq!(pong, "PONG");
                return Ok(());
            }
            if remaining != 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Err(format!("owned FalkorDB exhausted {attempts} readiness attempts").into())
    }

    pub(super) fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub(super) fn name(&self) -> &str {
        self.name
    }

    pub(super) async fn select_empty_graph(&mut self, name: &'static str) -> TestResult {
        self.name = name;
        let _: redis::Value = request(
            &self.endpoint,
            redis::cmd(inventory_graph::cypher::COMMAND)
                .arg(self.name)
                .arg("RETURN 1")
                .arg("TIMEOUT")
                .arg(10_000_u32),
            self.request_budget,
        )
        .await?;
        assert!(
            self.rows(inventory_graph::cypher::diagnostics::NODE_LABELS)
                .await?
                .is_empty(),
            "rebuild must begin in a verified empty second owned graph"
        );
        Ok(())
    }

    pub(super) async fn rows(&self, query: &str) -> TestResult<Vec<Vec<String>>> {
        let query = format!("{query} LIMIT {}", MAX_ROWS + 1);
        let value: redis::Value = request(
            &self.endpoint,
            redis::cmd("GRAPH.RO_QUERY")
                .arg(self.name)
                .arg(query)
                .arg("TIMEOUT")
                .arg(10_000_u32),
            self.request_budget,
        )
        .await?;
        let redis::Value::Array(parts) = value else {
            return Err("native graph response is not an array".into());
        };
        let Some(redis::Value::Array(rows)) = parts.get(1) else {
            return Err("native graph rows missing".into());
        };
        assert!(
            rows.len() <= MAX_ROWS,
            "exact graph evidence exceeded the explicit row budget"
        );
        rows.iter()
            .map(|row| {
                let redis::Value::Array(fields) = row else {
                    return Err("native graph row is not an array".into());
                };
                fields
                    .iter()
                    .map(|field| match field {
                        redis::Value::BulkString(value) => Ok(String::from_utf8(value.clone())?),
                        redis::Value::SimpleString(value) => Ok(value.clone()),
                        _ => Err(format!("native graph field is not text: {field:?}").into()),
                    })
                    .collect()
            })
            .collect()
    }

    pub(super) async fn stop(mut self) -> TestResult {
        self.terminate_child().await
    }

    async fn terminate_child(&mut self) -> TestResult {
        if self.child.try_wait()?.is_none() {
            self.child.start_kill()?;
        }
        tokio::time::timeout(Duration::from_secs(5), self.child.wait()).await??;
        Ok(())
    }
}

async fn request<T: redis::FromRedisValue>(
    endpoint: &str,
    command: &redis::Cmd,
    budget: Duration,
) -> TestResult<T> {
    // The one client deadline includes protocol setup, not only the final query.
    Ok(tokio::time::timeout(budget, async {
        let client = redis::Client::open(endpoint)?;
        let mut connection = client.get_multiplexed_async_connection().await?;
        command.query_async(&mut connection).await
    })
    .await??)
}

fn path_field(receipt: &serde_json::Value, field: &str) -> TestResult<PathBuf> {
    Ok(PathBuf::from(
        receipt
            .get(field)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("installer receipt lacks {field}"))?,
    ))
}

pub(super) async fn collect(native: &NativeRuntime, database: &Path, stage: &str) -> TestResult {
    let output = collect_at(
        native,
        database,
        stage,
        native.emulator().endpoint(),
        native.emulator().ca_bundle(),
    )
    .await?;
    if !output.status.success() {
        return Err(format!(
            "actual collector {stage} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}

pub(super) async fn collect_at(
    native: &NativeRuntime,
    database: &Path,
    stage: &str,
    endpoint: &str,
    ca: &Path,
) -> TestResult<std::process::Output> {
    let binary = std::env::var_os("INVENTORY_COLLECTOR_BINARY")
        .map(PathBuf::from)
        .ok_or("native graph evidence requires INVENTORY_COLLECTOR_BINARY built with aws")?;
    let stdout = native.work().join(format!("collector-{stage}.stdout"));
    let stderr = native.work().join(format!("collector-{stage}.stderr"));
    let mut command = Command::new(binary);
    command
        .env_clear()
        .envs(native.emulator().environment())
        .arg("sweep")
        .arg("--database")
        .arg(database)
        .arg("--catalog")
        .arg(runtime::catalog_path())
        .args(["--region", REGION])
        .arg("--endpoint")
        .arg(endpoint)
        .arg("--ca-bundle")
        .arg(ca)
        .args(["--max-pages", "10000", "--concurrency", "4"]);
    collector_process::run(&mut command, &stdout, &stderr, Duration::from_secs(600)).await
}

pub(super) async fn project(store: &BinaryStore, graph: &NativeGraph) -> TestResult {
    let backend = inventory_graph::falkor::FalkorDb::connect(graph.endpoint(), graph.name())?;
    for _ in 0..100 {
        let report = run_projection(store, &backend, 1000).await?;
        assert!(!report.offline, "native projection was offline: {report:?}");
        assert!(
            !report.rebuild_incomplete,
            "native rebuild incomplete: {report:?}"
        );
        if report.pending == 0 {
            assert_eq!(store.projection_state().await?.pending, 0);
            return Ok(());
        }
    }
    Err("native projection exceeded 100 batches of 1000".into())
}

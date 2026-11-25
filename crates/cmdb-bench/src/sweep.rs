pub(crate) mod args;
mod bounded;
mod collector;
mod machine;
mod plan;
mod proxy;
mod server;
mod signals;
mod summary;
mod table;
use crate::compare::{Row, Table};
use args::SweepArgs;
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_harness::{
    e2e::{self, stack::Stack},
    postgres::{TestDatabase, admin_url},
};
use sqlx::Connection;
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

struct Session<'a> {
    args: &'a SweepArgs,
    stack: &'a Stack,
    plan: plan::Plan,
    work: &'a Path,
    cancel: &'a CancellationToken,
}
struct Run {
    sample: table::Sample,
    extras: proxy::Extra,
    settings: Vec<(String, String)>,
    success: bool,
}

pub(crate) async fn run(mut args: SweepArgs) -> Result<bool, String> {
    args.collector = args.collector.canonicalize().map_err(text)?;
    args.catalog = args.catalog.canonicalize().map_err(text)?;
    let admin = admin_url().map_err(text)?;
    let temporary = if args.work.is_none() {
        Some(temporary_work()?)
    } else {
        None
    };
    let work = args
        .work
        .as_deref()
        .or_else(|| temporary.as_ref().map(tempfile::TempDir::path))
        .ok_or("missing work directory")?;
    e2e::prepare_work(work).map_err(text)?;
    let mut shutdown = signals::Shutdown::new()?;
    let cancel = shutdown.token();
    shutdown
        .run(async {
            let stack_work = work.to_path_buf();
            let stack = tokio::task::spawn_blocking(move || Stack::start(&stack_work))
                .await
                .map_err(text)?
                .map_err(text)?;
            let outcome = drive(&args, &stack, work, &admin, &cancel).await;
            let cleanup = tokio::task::spawn_blocking(move || stack.finish())
                .await
                .map_err(text)
                .and_then(|result| result.map_err(text));
            combine(outcome, cleanup)
        })
        .await
}
fn temporary_work() -> Result<tempfile::TempDir, String> {
    let home =
        std::env::var_os("HOME").ok_or("HOME required for default benchmark scratch directory")?;
    let cache = PathBuf::from(home).join(".cache/cmdb-bench");
    std::fs::create_dir_all(&cache).map_err(text)?;
    tempfile::Builder::new()
        .prefix("sweep-")
        .tempdir_in(cache)
        .map_err(text)
}
async fn drive(
    args: &SweepArgs,
    stack: &Stack,
    work: &Path,
    admin: &str,
    cancel: &CancellationToken,
) -> Result<bool, String> {
    if cancel.is_cancelled() {
        return Err("benchmark cancelled before seed".into());
    }
    let workspace = e2e::workspace_root();
    let plan = plan::plan(&workspace, args)?;
    let counts = tokio::select! {
        biased;
        () = cancel.cancelled() => return Err("benchmark cancelled during seed".into()),
        result = plan::seed(&plan, args, stack, work) => result?,
    };
    Session {
        args,
        stack,
        plan,
        work,
        cancel,
    }
    .measure(admin, counts)
    .await
}
impl Session<'_> {
    async fn measure(&self, admin: &str, counts: (usize, usize)) -> Result<bool, String> {
        let runs = stream::iter(0..self.args.runs)
            .map(Ok)
            .try_fold(Vec::new(), |mut runs, index| async move {
                runs.push(self.run_once(admin, index).await?);
                Ok::<_, String>(runs)
            })
            .await?;
        let first = runs.first().ok_or("no runs")?;
        let settings = first.settings.clone();
        if runs.iter().any(|run| run.settings != settings) {
            return Err("server settings changed during benchmark".into());
        }
        let success = runs.iter().all(|run| run.success);
        let extras = extra_rows(&runs)?;
        let unavailable = unavailable_headers(&runs);
        let samples = runs.into_iter().map(|run| run.sample).collect::<Vec<_>>();
        let mut rows = table::rows(&samples)?;
        rows.extend(extras);
        let mut headers = self.headers(settings, counts).await?;
        headers.extend(unavailable);
        headers.push((
            "collector.success".into(),
            if success { "yes" } else { "no" }.into(),
        ));
        headers.push(("protocol".into(), table::protocol(&samples)?.into()));
        let publishable = server::publishable(&headers);
        headers.push((
            "publishable".into(),
            if publishable {
                "yes"
            } else {
                "not publishable"
            }
            .into(),
        ));
        if !publishable {
            let reasons = headers
                .iter()
                .filter(|(key, _)| {
                    matches!(
                        key.as_str(),
                        "collector.success" | "commit_state" | "server.fsync" | "os"
                    )
                })
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("; ");
            headers.push(("publication_reason".into(), reasons));
        }
        table::publish(&Table { headers, rows }, self.args.out.as_deref()).await?;
        Ok(success)
    }
    async fn run_once(&self, admin: &str, index: usize) -> Result<Run, String> {
        let database = TestDatabase::create(admin).await.map_err(text)?;
        let outcome = self.sample(&database, index).await;
        let cleanup = database.finish().await.map_err(text);
        combine(outcome, cleanup)
    }
    async fn sample(&self, database: &TestDatabase, index: usize) -> Result<Run, String> {
        let mut connection = server::connect(database.url()).await?;
        let outcome = self
            .sample_connected(database, index, &mut connection)
            .await;
        let cleanup = connection.close().await.map_err(text);
        combine(outcome, cleanup)
    }
    async fn sample_connected(
        &self,
        database: &TestDatabase,
        index: usize,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Run, String> {
        let settings = server::settings(connection).await?;
        let before = server::counters(connection).await?;
        let (cold, extras) = self.collect(database, index, "cold").await?;
        let after = server::counters(connection).await?;
        let cold_database_bytes = server::database_bytes(connection).await?;
        let (warm, _) = self.collect(database, index, "warm").await?;
        let warm_counters = server::counters(connection).await?.since(after)?;
        let success = cold.code == 0 && warm.code == 0;
        let sample = table::Sample {
            cold_wall_ms: cold.wall_ms,
            warm_wall_ms: warm.wall_ms,
            cold: cold.lines,
            warm: warm.lines,
            cold_database_bytes,
            cold_wal_bytes: after.since(before)?.wal,
            warm_wal_bytes: warm_counters.wal,
            warm_rows_logged: warm_counters.logged,
            warm_rows_unlogged: warm_counters.unlogged,
        };
        Ok(Run {
            sample,
            extras,
            settings,
            success,
        })
    }
    async fn collect(
        &self,
        database: &TestDatabase,
        index: usize,
        phase: &str,
    ) -> Result<(collector::Finished, proxy::Extra), String> {
        let log = self.work.join(format!("run-{index}-{phase}.log"));
        match self.args.proxy {
            Some(spec) => {
                proxy::collect(
                    self.stack.emulator().endpoint(),
                    self.stack.emulator().ca_bundle(),
                    spec,
                    &self.work.join(format!("run-{index}-{phase}-proxy")),
                    async |front| {
                        front
                            .sweep(self.command(database, Some(front)), &log, self.cancel)
                            .await
                    },
                )
                .await
            }
            None => collector::sweep(self.command(database, None), &log, self.cancel)
                .await
                .map(|finished| (finished, Vec::new())),
        }
    }
    fn command(
        &self,
        database: &TestDatabase,
        front: Option<&proxy::Front>,
    ) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(&self.args.collector);
        self.stack.configure(&mut command);
        let (endpoint, ca) = front.map_or_else(
            || {
                (
                    self.stack.emulator().endpoint(),
                    self.stack.emulator().ca_bundle(),
                )
            },
            |proxy| (proxy.endpoint(), proxy.ca_bundle()),
        );
        command
            .arg("sweep")
            .env("INVENTORY_SWEEP_DATABASE", database.url())
            .arg("--catalog")
            .arg(&self.args.catalog)
            .args([
                "--profile",
                "default",
                "--region",
                "us-east-1",
                "--endpoint",
                endpoint,
            ])
            .arg("--ca-bundle")
            .arg(ca)
            .args([
                "--graph-url",
                self.stack.graph().url(),
                "--graph-name",
                database.name(),
                "--max-pages",
                "100000",
            ]);
        command.args(args::collector_options(self.args));
        self.plan.services.iter().for_each(|service| {
            command.args(["--service", service]);
        });
        command
    }
    async fn headers(
        &self,
        settings: Vec<(String, String)>,
        counts: (usize, usize),
    ) -> Result<Vec<(String, String)>, String> {
        let catalog = tokio::fs::read(&self.args.catalog).await.map_err(text)?;
        let mut headers = machine::headers(&e2e::workspace_root());
        let proxy = self.args.proxy.map_or_else(
            || "none".into(),
            |spec| format!("latency_ms={},rps={}", spec.latency_ms, spec.rps),
        );
        let max_rps = self
            .args
            .max_rps_milli
            .map_or_else(|| "adaptive".into(), args::rate_text);
        headers.extend([
            ("profile".into(), self.args.profile.name().into()),
            ("runs".into(), self.args.runs.to_string()),
            ("server_kind".into(), self.args.server.clone()),
            ("volume".into(), self.args.volume.clone()),
            ("build".into(), self.args.build.clone()),
            ("catalog_sha256".into(), machine::sha256_hex(&catalog)),
            (
                "emulator_version".into(),
                inventory_harness::emulator::PINNED_VERSION.into(),
            ),
            ("seed_resources_and_edges".into(), counts.0.to_string()),
            ("seed_steps".into(), counts.1.to_string()),
            ("services".into(), self.plan.services.join(",")),
            ("proxy".into(), proxy),
            (
                "concurrency".into(),
                args::setting_text(self.args.concurrency),
            ),
            (
                "idle_timeout_ms".into(),
                args::setting_text(self.args.idle_timeout_ms),
            ),
            (
                "pool_idle_per_host".into(),
                args::setting_text(self.args.pool_idle_per_host),
            ),
            ("max_rps".into(), max_rps),
            (
                "wal_scope".into(),
                "server-wide WAL; concurrent database writes may contribute".into(),
            ),
        ]);
        if self.args.proxy.is_some() {
            headers.push(("proxy_seed".into(), proxy::seed_header()));
            headers.push((
                "proxy_measurement_phase".into(),
                "cold; warm proxy extras discarded".into(),
            ));
            headers.push((
                "proxy_throughput_scope".into(),
                "admitted responses excluding throttles; per second, rounded down".into(),
            ));
            headers.push((
                "proxy_ramp_scope".into(),
                "aggregate admitted responses against one per-scope quota; not proof of scope saturation".into(),
            ));
        }
        headers.extend(settings);
        Ok(headers)
    }
}
fn unavailable_headers(runs: &[Run]) -> Vec<(String, String)> {
    runs.first().into_iter().flat_map(|run| &run.extras).filter_map(|(name, _)| {
        let missing = runs.iter().filter(|run| {
            run.extras.iter().find(|(key, _)| key == name).is_none_or(|(_, value)| value.is_none())
        }).count();
        (missing != 0).then(|| (format!("unavailable.{name}"), format!("{missing} runs: insufficient window, no responses, or threshold not reached")))
    }).collect()
}
fn extra_rows(runs: &[Run]) -> Result<Vec<Row>, String> {
    let names = runs
        .first()
        .ok_or("no runs")?
        .extras
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    names
        .into_iter()
        .filter_map(|name| {
            let values = runs
                .iter()
                .map(|run| {
                    run.extras
                        .iter()
                        .find(|(key, _)| *key == name)
                        .and_then(|(_, value)| *value)
                })
                .collect::<Option<Vec<_>>>();
            values.map(|values| table::spread(name, &values))
        })
        .collect()
}
fn combine<T>(outcome: Result<T, String>, cleanup: Result<(), String>) -> Result<T, String> {
    match (outcome, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}; cleanup: {cleanup}")),
    }
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
